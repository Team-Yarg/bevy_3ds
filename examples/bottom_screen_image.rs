use bevy::asset::AssetServer;
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::system::{Query, Res};
use bevy::input::Input;
use bevy::log::error;
use bevy::math::{Rect, Vec3};
use bevy::pbr::{PointLight, PointLightBundle};
use bevy::render::color::Color;
use bevy::render::texture::Image;
use bevy::scene::SceneBundle;
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup, Update},
    ecs::system::Commands,
};
use bevy_3ds_input::button::*;
use bevy_3ds_render::On3dsScreen;

mod setup_logger;
mod shims;

fn main() {
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();
    setup_logger::setup_logger().unwrap();
    {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            std::fs::write("panic.log", info.to_string()).expect("failed to write out panic log");
            error!("panic: {info}");
            prev(info)
        }));
    }

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn(SceneBundle {
        scene: assets.load("cornell-box.glb#Scene0"),
        ..Default::default()
    });

    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(On3dsScreen::Bottom);
    cmds.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.5, 0.5, 0.7),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.2),
        ..Default::default()
    });
}
