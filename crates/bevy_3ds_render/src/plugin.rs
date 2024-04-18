use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use std::time::Instant;

use bevy::asset::AssetLoader;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::ecs::system::SystemState;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::view::{
    ColorGrading, ExtractedView, NoFrustumCulling, RenderLayers, VisibilityPlugin, VisibleEntities,
};
use bevy::render::{color, primitives, Extract};
use bevy::time::TimeSender;
use bevy::{
    app::SubApp,
    ecs::{schedule::ScheduleLabel, system::Resource},
    prelude::*,
    render::{
        camera::CameraPlugin,
        mesh::{morph::MorphPlugin, MeshPlugin},
        view::{self},
        MainWorld, Render, RenderApp, RenderSet,
    },
};
use citro3d::render::{ClearFlags, Target};
use ctru::services::apt::Apt;
use ctru::services::gfx::{
    Gfx, RawFrameBuffer, Screen, Side, Swap, TopScreen3D, TopScreenLeft, TopScreenRight,
};

use crate::lighting::GpuLights;
use crate::{lighting, materials, On3dsScreen};

use super::draw::DrawCommands;
use super::pass::RenderPass;
use super::{mesh, shader, GfxInstance, GpuDevice, RenderSet3ds};

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
            materials::StandardMaterialPlugin,
            lighting::LightingRenderPlugin,
            ExtractComponentPlugin::<On3dsScreen>::default(),
        ));

        app.register_type::<color::Color>()
            .register_type::<primitives::Aabb>()
            .register_type::<primitives::CascadesFrusta>()
            .register_type::<primitives::CubemapFrusta>()
            .register_type::<primitives::Frustum>();
    }
}

#[derive(Default, Resource)]
struct ScratchMainWorld(MainWorld);

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
        .add_systems(
            Render,
            (
                apply_extract_commands.in_set(RenderSet::ExtractCommands),
                render_system.in_set(RenderSet::Render),
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

fn render_system(world: &mut World) {
    log::debug!("render on thread {:?}", std::thread::current().id());
    #[allow(clippy::type_complexity)]
    let mut st: SystemState<(
        Res<GpuDevice>,
        NonSend<GfxInstance>,
        Res<DrawCommands>,
        Res<ClearColor>,
        Query<(&ExtractedCamera, &ExtractedView, Option<&On3dsScreen>)>,
        Res<GpuLights>,
    )> = SystemState::new(world);
    let (gpu, gfx, commands, clear_colour, cameras, lights) = st.get(world);
    let gpu = gpu.into_inner();
    gfx.0.wait_for_vblank();

    let use_3d = cameras.iter().any(|c| {
        c.2.is_some_and(|s| {
            if let On3dsScreen::Top(t) = s {
                t.is_some()
            } else {
                false
            }
        })
    });

    let top_screen_3d: Option<TopScreen3D> = if use_3d {
        Some((&gfx.0.top_screen).into())
    } else {
        None
    };

    let (top_screen_left, top_screen_right) =
        match top_screen_3d.as_ref().map(TopScreen3D::split_mut) {
            None => (None, None),
            Some((l, r)) => (Some(l), Some(r)),
        };

    fn create_target<'a>(
        mut screen: RefMut<'a, dyn Screen>,
        clear_colour: &ClearColor,
    ) -> Target<'a> {
        let RawFrameBuffer { width, height, .. } = screen.raw_framebuffer();
        let mut target = citro3d::render::Target::new(
            width,
            height,
            screen,
            Some(citro3d::render::DepthFormat::Depth16),
        )
        .expect("failed to create render target");
        target
    }

    let mut targets = if use_3d {
        [
            Some(create_target(
                gfx.0.bottom_screen.borrow_mut() as _,
                &clear_colour,
            )),
            Some(create_target(top_screen_left.unwrap() as _, &clear_colour)),
            Some(create_target(top_screen_right.unwrap() as _, &clear_colour)),
        ]
    } else {
        [
            Some(create_target(
                gfx.0.bottom_screen.borrow_mut() as _,
                &clear_colour,
            )),
            Some(create_target(
                gfx.0.top_screen.borrow_mut() as _,
                &clear_colour,
            )),
            None,
        ]
    };

    fn set_render_target<'a>(
        ty: Option<On3dsScreen>,
        gfx: &'a Gfx,
        pass: &mut RenderPass,
        targets: &mut [Option<Target<'a>>],
        side: Option<Side>,
    ) {
        let ty = ty.unwrap_or_default();
        let target_idx = ty.to_target_index(side);
        pass.select_render_target(targets[target_idx].as_ref().unwrap());
    }

    {
        let frame: crate::frame::Citro3dFrame<'_> = gpu.start_new_frame();
        let mut pass = RenderPass::new(gpu, &frame);
        commands.prepare(world);

        for t in &mut targets {
            if let Some(ta) = t {
                ta.clear(
                    ClearFlags::ALL,
                    clear_colour.as_linear_rgba_u32().to_be(),
                    0,
                );
            }
        }

        for (_, view, ty) in &cameras {
            let view_mtx = view.transform.compute_matrix().inverse();

            if use_3d {
                match ty.copied().unwrap_or_default() {
                    On3dsScreen::Bottom => {
                        set_render_target(ty.copied(), &gfx.0, &mut pass, &mut targets, None);
                        pass.set_light_positions(&lights.lights, view_mtx);

                        commands
                            .run(world, &mut pass, view)
                            .expect("failed to run draws");
                    }

                    On3dsScreen::Top(None) => {
                        pass.set_light_positions(&lights.lights, view_mtx);

                        set_render_target(
                            ty.copied(),
                            &gfx.0,
                            &mut pass,
                            &mut targets,
                            Some(Side::Left),
                        );
                        commands
                            .run(world, &mut pass, view)
                            .expect("failed to run left draws");

                        set_render_target(
                            ty.copied(),
                            &gfx.0,
                            &mut pass,
                            &mut targets,
                            Some(Side::Right),
                        );
                        commands
                            .run(world, &mut pass, view)
                            .expect("failed to run right draws");
                    }

                    On3dsScreen::Top(Some(f)) => {
                        let (left_view, right_view) = f(view);
                        let left_mtx = left_view.transform.compute_matrix().inverse();
                        let right_mtx = right_view.transform.compute_matrix().inverse();

                        set_render_target(
                            ty.copied(),
                            &gfx.0,
                            &mut pass,
                            &mut targets,
                            Some(Side::Left),
                        );
                        pass.set_light_positions(&lights.lights, left_mtx);
                        commands
                            .run(world, &mut pass, &left_view)
                            .expect("failed to run left draws");

                        set_render_target(
                            ty.copied(),
                            &gfx.0,
                            &mut pass,
                            &mut targets,
                            Some(Side::Right),
                        );
                        pass.set_light_positions(&lights.lights, right_mtx);
                        commands
                            .run(world, &mut pass, &right_view)
                            .expect("failed to run right draws");
                    }
                }
            } else {
                set_render_target(ty.copied(), &gfx.0, &mut pass, &mut targets, None);

                pass.set_light_positions(&lights.lights, view_mtx);

                commands
                    .run(world, &mut pass, view)
                    .expect("failed to run draws");
            }
        }
    }

    log::debug!("render fin");

    let time_send = world.resource::<TimeSender>();
    if let Err(e) = time_send.0.try_send(Instant::now()) {
        match e {
            bevy::time::TrySendError::Full(_) => {
                panic!("The TimeSender channel should always be empty during render. You might need to add the bevy::core::time_system to your app.",);
            }
            bevy::time::TrySendError::Disconnected(_) => {}
        }
    }
}

fn apply_extract_commands(render_world: &mut World) {
    render_world.resource_scope(|render_world, mut schedules: Mut<Schedules>| {
        schedules
            .get_mut(ExtractSchedule)
            .unwrap()
            .apply_deferred(render_world);
    });
}
