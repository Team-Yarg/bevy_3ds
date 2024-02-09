use std::f32::consts::PI;

use bevy::app::Update;
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res};
use bevy::input::Axis;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::render::color::Color;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::{CompressedImageFormats, Image};
use bevy::scene::{Scene, SceneBundle};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    hierarchy::BuildChildren,
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        Style, Val,
    },
};
use bevy_3ds_input::axis::{Axis3ds, Axis3dsType};

mod shims;

#[derive(Component)]
struct CornellBox;

fn main() {
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, update);

    app.run();
}
fn update(
    time: Res<Time>,
    mut s: Query<(&mut Transform), With<CornellBox>>,
    axis: Res<Axis<Axis3ds>>,
) {
    let velocity = PI / 4.0;
    let mut x_velocity: f32 = velocity;
    let mut y_velocity: f32 = velocity;
    if let Some(x) = axis.get(Axis3ds::new(Axis3dsType::CPadX)) {
        x_velocity *= x;
    }

    if let Some(y) = axis.get(Axis3ds::new(Axis3dsType::CPadY)) {
        y_velocity *= y;
    }
    for mut t in &mut s {
        t.rotate_around(
            Vec3::ZERO,
            Quat::from_rotation_y(x_velocity * time.delta_seconds()),
        );
        t.rotate_around(
            Vec3::ZERO,
            Quat::from_rotation_x(y_velocity * time.delta_seconds()),
        );
    }
}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn(SceneBundle {
        scene: assets.load("cornell-box.glb#Scene0"),
        ..Default::default()
    })
    .insert(CornellBox);

    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
