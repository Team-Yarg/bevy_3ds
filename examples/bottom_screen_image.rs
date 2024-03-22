use bevy::asset::AssetServer;
use bevy::ecs::system::{Query, Res};
use bevy::input::Input;
use bevy::log::error;
use bevy::math::{Rect, Vec2};
use bevy::render::color::Color;
use bevy::render::texture::Image;
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
};
use bevy_3ds::render::BottomScreenTexture;
use bevy_3ds_input::button::*;
use bevy_3ds_render::On3dsScreen;

mod setup_logger;
mod shims;

const IMG_BYTES: &[u8] = include_bytes!("../romfs/assets/controls-1.png");

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
    let birb = assets.load("bevy_bird.png");
    cmds.spawn(SpriteBundle {
        sprite: Sprite {
            //custom_size: Some(Vec2::new(320.0, 240.0)),
            rect: Some(Rect::new(0.0, 0.0, 320.0, 240.0)),
            ..Default::default()
        },
        texture: birb,
        ..Default::default()
    });
    cmds.spawn(Camera2dBundle::default())
        .insert(On3dsScreen::Bottom);
}
