//! # Bevy Game Example
//!
//! This module demonstrates a simple Bevy game application, including setup & sprite movement.

#![feature(allocator_api)]

use bevy::asset::AssetServer;
use bevy::ecs::system::{Query, Res};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::texture::{CompressedImageFormats, Image};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    hierarchy::BuildChildren,
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        Style, Val,
    },
};

use tracing::error;

mod shims;

//use libc::c_void;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%+"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::fs::File::create("output.log")?)
        .apply()?;
    Ok(())
}

/// Sets up the Bevy application.
#[cfg(target_os = "horizon")]
fn ds_main() {
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let mut app = App::new();
    app.add_plugins((
        bevy_3ds::DefaultPlugins,
        bevy_3ds_input::test::Input3dsTestPlugin,
    ));
    app.add_systems(Startup, setup);
    app.add_systems(Update, pupdate);

    app.run();
}

/// Main entry point for the application.
/// Configures panic hook and logger, then calls `ds_main`.
fn main() {
    {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            error!("{}", info);
            prev(info);
        }));
    }
    //#[cfg(debug)]
    //{
    setup_logger().expect("failed to init logger");
    log::set_max_level(log::LevelFilter::Debug); // this prevents evaluating log statements below, which fern doesn't do
                                                 //}

    ds_main();
}

use bevy::input::Input;
use bevy_3ds_input::button::*;
/// Update function for sprite movement.
/// Moves each sprite in the `sprites` query to the right each frame.
fn pupdate(mut sprites: Query<(&Sprite, &mut Transform)>, buttons: Res<Input<Button3ds>>) {
    for (_, mut pos) in &mut sprites {
        let d = 10.0;
        if buttons.just_pressed(Button3ds::new(Button3dsType::DPadLeft)) {
            pos.translation.x -= d;
        }

        if buttons.just_pressed(Button3ds::new(Button3dsType::DPadRight)) {
            pos.translation.x += d;
        }

        if buttons.just_pressed(Button3ds::new(Button3dsType::DPadUp)) {
            pos.translation.y += d;
        }

        if buttons.just_pressed(Button3ds::new(Button3dsType::DPadDown)) {
            pos.translation.y -= d;
        }

        if pos.translation.x > 32. {
            pos.translation.x = -32.;
        }
    }
}

/// Setup function for initialising game entities.
/// Loads assets, creates a sprite, and sets up the camera and UI.
fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    let img_bytes = include_bytes!("../romfs/assets/peach.png");
    let _img = Image::from_buffer(
        img_bytes,
        bevy::render::texture::ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        bevy::render::texture::ImageSampler::Default,
    )
    .unwrap();
    //let peach = assets.add(img);
    let peach = assets.load("peach.png");

    let _tri = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
        )
        .with_indices(Some(bevy::render::mesh::Indices::U32(vec![
            0, 3, 1, 1, 3, 2,
        ])));

    cmds.spawn(PbrBundle {
        mesh: assets.add(_tri),
        material: assets.add(StandardMaterial {
            base_color_texture: Some(peach.clone()),
            ..Default::default()
        }),
        ..Default::default()
    });

    cmds.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgba(1.0, 0.5, 0.5, 1.0),
            ..Default::default()
        },
        texture: peach,
        ..Default::default()
    });
    cmds.spawn(Camera2dBundle::default());
    cmds.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section("Hello, World", Default::default()));
    });
}
