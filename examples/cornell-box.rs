use std::f32::consts::PI;

use bevy::app::Update;
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res};
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial};
use bevy::render::color::Color;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::{CompressedImageFormats, Image};
use bevy::render::view::ExtractedView;
use bevy::scene::{Scene, SceneBundle};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    hierarchy::BuildChildren,
};
use bevy_3ds_render::{pending_render_system, CameraID, On3dsScreen, RenderOn};

mod shims;

#[derive(Component)]
struct CornellBox;

fn main() {
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, update);
    app.add_systems(Update, pending_render_system);

    app.run();
}
fn update(time: Res<Time>, mut s: Query<(&mut Transform), With<CornellBox>>) {
    let velocity = PI / 4.0;
    for mut t in &mut s {
        t.rotate_around(
            Vec3::ZERO,
            Quat::from_rotation_y(velocity * time.delta_seconds()),
        );
    }
}

// this should maybe be moved somewhere else
fn copy_view(view: &ExtractedView, mul_by: Transform) -> ExtractedView {
    ExtractedView {
        projection: view.projection.clone(),
        transform: view.transform.clone() * mul_by,
        view_projection: view.view_projection.clone(),
        hdr: view.hdr.clone(),
        viewport: view.viewport.clone(),
        color_grading: view.color_grading.clone(),
    }
}

fn stereo_displacement(view: &ExtractedView) -> Option<(ExtractedView, ExtractedView)> {
    let slider_val = ctru::os::current_3d_slider_state();

    #[allow(clippy::float_cmp)]
    if slider_val == 0.0 {
        // uncomment this to disable drawing when the slider is at 0
        //return None;
    }

    let interocular_distance = slider_val / 2.0;
    let displacement = interocular_distance / 2.0;

    let left = Transform::from_translation(Vec3::new(-displacement, 0.0, 0.0));
    let right = Transform::from_translation(Vec3::new(displacement, 0.0, 0.0));

    Some((copy_view(view, left), copy_view(view, right)))
}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn(SceneBundle {
        scene: assets.load("cornell-box.glb#Scene0"),
        ..Default::default()
    })
    .insert(CornellBox)
    .insert(RenderOn::Only(0.into()).make_pending());

    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(On3dsScreen::Top(Some(stereo_displacement)));
    cmds.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.5, 0.5, 0.7),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.2),
        ..Default::default()
    });

    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(On3dsScreen::Bottom)
    .insert(CameraID::from(1));
}
