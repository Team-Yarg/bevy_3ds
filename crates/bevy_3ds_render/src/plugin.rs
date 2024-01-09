use std::sync::{Arc, Mutex};

use std::ops::Deref;

use bevy::asset::AssetLoader;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d::Core2dPlugin;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::ecs::system::SystemState;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_resource::ShaderLoaderError;
use bevy::render::view::{
    ColorGrading, NoFrustumCulling, RenderLayers, VisibilityPlugin, VisibleEntities,
};
use bevy::sprite::ExtractedSprites;
use bevy::{
    app::{App, Plugin, SubApp},
    ecs::{
        schedule::{Schedule, ScheduleLabel},
        system::{Query, Resource},
        world::World,
    },
    prelude::*,
    render::{
        camera::CameraPlugin,
        globals::GlobalsPlugin,
        mesh::{morph::MorphPlugin, MeshPlugin},
        render_resource::ShaderLoader,
        view::{self, ViewPlugin},
        ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
    },
    transform::components::GlobalTransform,
    ui::{
        node_bundles::NodeBundle, BackgroundColor, ExtractedUiNode, ExtractedUiNodes,
        RenderUiSystem,
    },
    DefaultPlugins,
};
use citro3d::render::ClearFlags;
use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen, TopScreen3D};
use ctru::{
    console::Console,
    services::{apt::Apt, gfx::Gfx},
};

use super::draw::DrawCommands;
use super::pass::RenderPass;
use super::prep_asset::RenderAssets;
use super::{mesh, shader, texture, GfxInstance, GpuDevice, RenderSet3ds};

struct AptRes(Apt);

impl Resource for AptRes {}

impl Default for AptRes {
    fn default() -> Self {
        Self(Apt::new().unwrap())
    }
}

pub struct Render3dsPlugin;

#[derive(Default, Debug)]
struct WgpuShaderLoaderDummy;

#[derive(thiserror::Error, Debug)]
#[error("wgsl shaders are disabled (you're on 3ds)")]
struct WgpuShaderLoadDummyError;

impl AssetLoader for WgpuShaderLoaderDummy {
    type Asset = Shader;
    type Settings = ();
    type Error = WgpuShaderLoadDummyError;

    fn load<'a>(
        &'a self,
        _reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        log::debug!("intercepted shader load for {}", load_context.asset_path());
        Box::pin(async move { Err(WgpuShaderLoadDummyError) })
    }

    fn extensions(&self) -> &[&str] {
        &["spv", "wgsl", "vert", "frag", "comp"]
    }
}

struct ViewPlugin3ds;

impl Plugin for ViewPlugin3ds {
    fn build(&self, app: &mut App) {
        app.register_type::<InheritedVisibility>()
            .register_type::<ViewVisibility>()
            .register_type::<Msaa>()
            .register_type::<NoFrustumCulling>()
            .register_type::<RenderLayers>()
            .register_type::<Visibility>()
            .register_type::<VisibleEntities>()
            .register_type::<ColorGrading>()
            .init_resource::<Msaa>()
            // NOTE: windows.is_changed() handles cases where a window was resized
            .add_plugins((ExtractResourcePlugin::<Msaa>::default(), VisibilityPlugin));
    }
}

struct Core3dPlugin;

impl Plugin for Core3dPlugin {
    fn build(&self, app: &mut App) {
        use bevy::core_pipeline::core_3d::Camera3dDepthLoadOp;
        app.register_type::<Camera3d>()
            .register_type::<Camera3dDepthLoadOp>()
            .add_plugins(ExtractComponentPlugin::<Camera3d>::default());
    }
}

pub struct CorePipeline3ds;
impl Plugin for CorePipeline3ds {
    fn build(&self, app: &mut App) {
        app.register_type::<ClearColor>()
            .register_type::<ClearColorConfig>()
            .register_type::<DepthPrepass>()
            .register_type::<NormalPrepass>()
            .init_resource::<ClearColor>()
            .add_plugins((
                ExtractResourcePlugin::<ClearColor>::default(),
                Core2dPlugin,
                Core3dPlugin,
            ));
    }
}

impl Plugin for Render3dsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Shader>()
            .init_asset_loader::<WgpuShaderLoaderDummy>();
        init_render_app(app);
        app.add_plugins((
            ValidParentCheckPlugin::<view::InheritedVisibility>::default(),
            // todo: window plugin
            CameraPlugin,
            ViewPlugin3ds, // view plugin
            MeshPlugin,
            mesh::MeshPlugin,
            MorphPlugin,
            shader::PicaShaderPlugin,
        ));
    }
}

#[derive(Default, Resource)]
struct ScratchMainWorld(MainWorld);

impl ScratchMainWorld {
    fn take(self) -> World {
        let w: &World = self.0.deref();
        unsafe { std::ptr::read(w as *const World) }
    }
}

fn init_render_app(parent: &mut App) {
    parent.init_resource::<ScratchMainWorld>();

    let mut extract_schedule = Schedule::new(ExtractSchedule);
    extract_schedule.set_apply_final_deferred(false);

    let mut app = App::empty();
    app.main_schedule_label = Render.intern();
    let mut base_shed = Render::base_schedule();
    base_shed.configure_sets(
        (
            RenderSet::Queue,
            RenderSet::QueueMeshes,
            RenderSet::ManageViews,
            RenderSet::ManageViewsFlush,
            RenderSet::Prepare,
            RenderSet::PrepareAssets,
            RenderSet::PrepareResources,
            RenderSet::PrepareResourcesFlush,
            RenderSet::PrepareBindGroups,
            RenderSet::PrepareFlush,
        )
            .run_if(|| false),
    );
    base_shed.configure_sets(
        (
            RenderSet::ExtractCommands,
            RenderSet3ds::PrepareAssets,
            RenderSet3ds::Prepare,
            RenderSet3ds::PrepareBindGroups,
        )
            .chain(),
    );

    app.add_schedule(extract_schedule)
        .add_schedule(base_shed)
        .init_resource::<bevy::render::render_graph::RenderGraph>()
        .init_resource::<GpuDevice>()
        .init_resource::<DrawCommands>()
        .init_non_send_resource::<GfxInstance>()
        .insert_resource(parent.world.resource::<bevy::asset::AssetServer>().clone())
        .add_systems(Render, render_system)
        .add_systems(
            Render,
            (
                apply_extract_commands.in_set(RenderSet::ExtractCommands),
                (render_system, render_ui, render_meshes, render_sprites).in_set(RenderSet::Render),
                World::clear_entities.in_set(RenderSet::Cleanup),
            ),
        );

    let (sender, receiver) = bevy::time::create_time_channels();
    parent.insert_resource(receiver);
    app.insert_resource(sender);

    parent.insert_sub_app(RenderApp, SubApp::new(app, move |main_world, render_app| {
            // reserve all existing main world entities for use in render_app
            // they can only be spawned using `get_or_spawn()`
            let total_count = main_world.entities().total_count();

            assert_eq!(
                render_app.world.entities().len(),
                0,
                "An entity was spawned after the entity list was cleared last frame and before the extract schedule began. This is not supported",
            );

            // This is safe given the clear_entities call in the past frame and the assert above
            unsafe {
                render_app
                    .world
                    .entities_mut()
                    .flush_and_reserve_invalid_assuming_no_entities(total_count);
            }
        // run extract schedule
        extract(main_world, render_app);
    }));
}

/// Executes the [`ExtractSchedule`] step of the renderer.
/// This updates the render world with the extracted ECS data of the current frame.
fn extract(main_world: &mut World, render_app: &mut App) {
    println!("extract");
    // temporarily add the app world to the render world as a resource
    let mut scratch_world = main_world.remove_resource::<ScratchMainWorld>().unwrap();
    std::mem::swap(main_world, &mut scratch_world.0);

    render_app.world.insert_resource(scratch_world.0);

    render_app.world.run_schedule(ExtractSchedule);

    // move the app world back, as if nothing happened.
    let mut inserted_world = render_app.world.remove_resource::<MainWorld>().unwrap();
    std::mem::swap(main_world, &mut inserted_world);
    main_world.insert_resource(ScratchMainWorld(inserted_world));
}
fn render_ui(nodes: Res<ExtractedUiNodes>) {
    //println!("render ui");
    for (ent, node) in &nodes.uinodes {
        println!("node: {ent:#?}");
    }
}
fn render_meshes(meshes: Res<RenderAssets<Mesh>>) {
    println!("render meshes: {}", meshes.iter().count());
    for (id, mesh) in meshes.iter() {
        println!("render mesh: {id:#?}");
    }
}
fn render_sprites(sprites: Res<ExtractedSprites>) {
    log::debug!("sprites: {}", sprites.sprites.len());
}

fn render_system(world: &mut World) {
    log::debug!("render");
    #[allow(clippy::type_complexity)]
    let mut st: SystemState<(
        Res<GpuDevice>,
        NonSend<GfxInstance>,
        Res<DrawCommands>,
        Query<(Entity, &ExtractedCamera)>,
    )> = SystemState::new(world);
    let (gpu, gfx, commands, cameras) = st.get(world);
    let gpu = gpu.into_inner();
    let mut screen = gfx.0.top_screen.borrow_mut();
    let RawFrameBuffer { width, height, .. } = screen.raw_framebuffer();

    let mut target = citro3d::render::Target::new(
        width,
        height,
        screen,
        Some(citro3d::render::DepthFormat::Depth16),
    )
    .expect("failed to create left render target");
    target.clear(ClearFlags::ALL, 0xFFFFFF, 0);

    let mut pass = RenderPass::new(gpu).expect("failed to create render pass");
    pass.select_render_target(&target);
    commands.prepare(world);

    for (id, _) in &cameras {
        commands
            .run(world, &mut pass, id)
            .expect("failed to run draws");
    }

    drop(pass);
    log::debug!("render fin");
}

fn apply_extract_commands(render_world: &mut World) {
    render_world.resource_scope(|render_world, mut schedules: Mut<Schedules>| {
        schedules
            .get_mut(ExtractSchedule)
            .unwrap()
            .apply_deferred(render_world);
    });
}