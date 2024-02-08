//! # Bevy Game Example
//!
//! This module demonstrates a simple Bevy game application, including setup & sprite movement.

use bevy::asset::AssetServer;
use bevy::ecs::system::{Query, Res};
use bevy::input::Input;
use bevy::math::Vec2;
use bevy::render::color::Color;
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
};
use bevy_3ds_input::button::*;

use tracing::error;

mod shims;

fn main() {
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, pupdate);

    app.run();
}

/// Update function for sprite movement.
/// Moves each sprite in the `sprites` query to the right each frame.
fn pupdate(mut sprites: Query<(&Sprite, &mut Transform)>, buttons: Res<Input<Button3ds>>) {
    for (_, mut pos) in &mut sprites {
        let d = 10.0;
        if buttons.pressed(Button3ds::new(Button3dsType::DPadLeft)) {
            pos.translation.x -= d;
        }

        if buttons.pressed(Button3ds::new(Button3dsType::DPadRight)) {
            pos.translation.x += d;
        }

        if buttons.pressed(Button3ds::new(Button3dsType::DPadUp)) {
            pos.translation.y += d;
        }

        if buttons.pressed(Button3ds::new(Button3dsType::DPadDown)) {
            pos.translation.y -= d;
        }
    }
}

/// Setup function for initialising game entities.
/// Loads assets, creates a sprite, and sets up the camera and UI.
fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    let birb = assets.load("bevy_bird.png");

    cmds.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgba(1.0, 0.5, 0.5, 1.0),
            custom_size: Some(Vec2::splat(64.0)),
            ..Default::default()
        },
        texture: birb,
        ..Default::default()
    });
    cmds.spawn(Camera2dBundle::default());
}
