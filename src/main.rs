#![feature(allocator_api)]

use std::alloc::Layout;
use std::io::Write;
use std::time::SystemTime;
use std::{fs::File, panic::PanicInfo};

use bevy::asset::{AssetEvent, AssetServer, Assets};
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::event::EventReader;
use bevy::ecs::schedule::{Schedule, ScheduleGraph};
use bevy::ecs::system::{Query, Res, ResMut};
use bevy::math::Vec2;
use bevy::render::camera::OrthographicProjection;
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::texture::{CompressedImageFormats, Image, ImageLoader};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::utils::hashbrown::{HashMap, HashSet};
use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    hierarchy::BuildChildren,
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        Style, Val,
    },
    DefaultPlugins,
};
#[cfg(target_os = "horizon")]
use ctru::services::{
    self,
    apt::Apt,
    gfx::Gfx,
    hid::{Hid, KeyPad},
};
use tracing::{debug, error};

mod shims;

//use libc::c_void;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%+"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::fs::File::create("output.log")?)
        .apply()?;
    Ok(())
}

#[cfg(target_os = "horizon")]
fn ds_main() {
    use bevy::app::PostUpdate;

    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(PostUpdate, pupdate);

    app.run();
}

fn main() {
    {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            error!("{}", info);
            prev(info);
        }));
    }
    setup_logger().expect("failed to init logger");

    ds_main();
}

fn noop(mut cmds: Commands) {}
fn pupdate(mut sprites: Query<(&Sprite, &mut Transform)>) {
    for (_, mut pos) in &mut sprites {
        pos.translation.x += 1.0;
        if pos.translation.x > 32. {
            pos.translation.x = 0.;
        }
    }
}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    let img_bytes = include_bytes!("../romfs/assets/peach.png");
    let img = Image::from_buffer(
        img_bytes,
        bevy::render::texture::ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        bevy::render::texture::ImageSampler::Default,
    )
    .unwrap();
    //let peach = assets.add(img);
    let peach = assets.load("peach.png");

    let tri = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
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
