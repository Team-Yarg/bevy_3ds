use std::sync::{Arc, Mutex};

use std::ops::Deref;

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
use ctru::{
    console::Console,
    services::{apt::Apt, gfx::Gfx},
};

use super::RenderSet3ds;

struct AptRes(Apt);

impl Resource for AptRes {}

impl Default for AptRes {
    fn default() -> Self {
        Self(Apt::new().unwrap())
    }
}

pub struct Render3dsPlugin;

impl Plugin for Render3dsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.set_runner(move |mut app| {
            let apt = Apt::new().unwrap();
            println!("run");
            //let gfx = Gfx::new().unwrap();
            while apt.main_loop() {
                //gfx.wait_for_vblank();
                app.update();
            }
        });
        app.init_asset::<Shader>()
            .init_asset_loader::<ShaderLoader>();
        init_render_app(app);
        app.add_plugins((
            ValidParentCheckPlugin::<view::InheritedVisibility>::default(),
            CameraPlugin,
            ViewPlugin,
            MeshPlugin,
            GlobalsPlugin,
            MorphPlugin,
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
    base_shed.configure_sets((RenderSet::ExtractCommands, RenderSet3ds::PrepareAssets).chain());

    app.add_schedule(extract_schedule)
        .add_schedule(base_shed)
        .init_resource::<bevy::render::render_graph::RenderGraph>()
        .insert_resource(parent.world.resource::<bevy::asset::AssetServer>().clone())
        .add_systems(
            Render,
            (
                apply_extract_commands.in_set(RenderSet::ExtractCommands),
                (render_system, render_ui).in_set(RenderSet::Render),
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
    println!("render ui");
    for (ent, node) in &nodes.uinodes {
        println!("node: {ent:#?}");
    }
}

fn render_system(world: &mut World) {
    println!("render");
    draw_triangle();
}

fn apply_extract_commands(render_world: &mut World) {
    render_world.resource_scope(|render_world, mut schedules: Mut<Schedules>| {
        schedules
            .get_mut(ExtractSchedule)
            .unwrap()
            .apply_deferred(render_world);
    });
}

fn draw_triangle() {}
