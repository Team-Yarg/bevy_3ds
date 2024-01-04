#![feature(allocator_api)]

use std::alloc::Layout;
use std::io::Write;
use std::time::SystemTime;
use std::{fs::File, panic::PanicInfo};

use bevy::asset::{AssetEvent, AssetServer, Assets};
use bevy::ecs::event::EventReader;
use bevy::ecs::schedule::{Schedule, ScheduleGraph};
use bevy::ecs::system::{Res, ResMut};
use bevy::math::Vec2;
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::sprite::{Sprite, SpriteBundle};
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

#[cfg(target_os = "horizon")]
mod romfs_assets;

#[cfg(target_os = "horizon")]
mod ui;

#[cfg(target_os = "horizon")]
mod sprites;

#[cfg(target_os = "horizon")]
mod render;

#[cfg(target_os = "horizon")]
mod gpu_buffer;

#[cfg(target_os = "horizon")]
mod shims;

#[cfg(target_os = "horizon")]
mod plugin_3ds;

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
        .level(log::LevelFilter::Debug)
        .chain(std::fs::File::create("output.log")?)
        .apply()?;
    Ok(())
}

#[cfg(target_os = "horizon")]
fn ds_main() {
    use std::{cell::Cell, hash::RandomState};

    use bevy::{
        asset::AssetPlugin,
        core_pipeline::CorePipelinePlugin,
        hierarchy::HierarchyPlugin,
        log::LogPlugin,
        render::{render_resource::RenderPipeline, texture::ImagePlugin},
        sprite::SpritePlugin,
        text::TextPlugin,
        transform::TransformPlugin,
        ui::UiPlugin,
        window::{PrimaryWindow, Window, WindowPlugin, WindowResolution},
        MinimalPlugins,
    };
    use indexmap::IndexMap;

    use crate::render::Render3dsPlugin;
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    /*let gfx = Gfx::new().unwrap();
    let tty = ctru::console::Console::new(gfx.bottom_screen.borrow_mut());*/

    std::env::set_var("BEVY_ASSET_ROOT", "romfs:/");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((TransformPlugin, HierarchyPlugin, bevy::input::InputPlugin))
        .add_plugins(romfs_assets::RomfsAssetPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(Render3dsPlugin)
        .add_plugins(ImagePlugin::default())
        .add_plugins(CorePipelinePlugin)
        .add_plugins((
            SpritePlugin,
            TextPlugin,
            WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(480., 240.),
                    resizable: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
        ));
    app.add_plugins(UiPlugin);
    app.add_systems(Startup, setup);
    println!("hello");

    app.run();
}
#[cfg(not(target_os = "horizon"))]
fn ds_main() {
    let mut app = App::empty();
    let mut shed = ScheduleGraph::new();

    let mut v = Vec::with_capacity(0);
    //let mut v = Vec::new();
    v.push(5);
    println!("v: {v:#?}");
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let mut f = File::create("panics.log").unwrap();
        write!(f, "{}", info).ok();
    }));
    setup_logger().expect("failed to init logger");

    /*app.set_runner(move |mut app| {
        while apt.main_loop() {
            //gfx.wait_for_vblank();
            app.update();
        }
    });*/
    //app.add_plugins(DefaultPlugins);
    //let mut m0 = HashMap::<i8, char>::new();
    //let mut m = HashSet::<char>::new();
    //shed.set_executor_kind(bevy::ecs::schedule::ExecutorKind::Simple);
    //app.add_schedule(shed);
    //app.add_systems(Startup, setup);
    //app.add_systems(Startup, noop);
    //app.run();
    ds_main();
}

fn noop(mut cmds: Commands) {}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
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
        texture: assets.load("assets/peach.png"),
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

/*
#[no_mangle]
pub unsafe extern "C" fn getrandom(
    buf: *mut libc::c_void,
    mut buflen: libc::size_t,
    flags: libc::c_uint,
) -> libc::ssize_t {
    // Based on https://man7.org/linux/man-pages/man2/getrandom.2.html
    // Technically we only have one source (no true /dev/random), but the
    // behavior should be more expected this way.
    let maxlen = if flags & libc::GRND_RANDOM != 0 {
        512
    } else {
        0x1FFFFFF
    };
    buflen = buflen.min(maxlen);

    // Avoid conflicting a real POSIX errno by using a value < 0
    // Should we define this in ctru-sys somewhere or something?
    const ECTRU: libc::c_int = -1;

    let ret = ctru_sys::psInit();

    // Error handling code for psInit
    if ctru_sys::R_FAILED(ret) {
        // Best-effort attempt at translating return codes
        *__errno() = match ctru_sys::R_SUMMARY(ret) as libc::c_uint {
            // The service handle is full (would block to await availability)
            ctru_sys::RS_WOULDBLOCK => libc::EAGAIN,
            // The caller doesn't have the right to call the service
            _ => ECTRU,
        };
        return -1;
    }

    let ret = ctru_sys::PS_GenerateRandomBytes(buf, buflen);

    // Error handling code for PS_GenerateRandomBytes
    if ctru_sys::R_SUCCEEDED(ret) {
        // Safe because above ensures buflen < isize::MAX
        buflen as libc::ssize_t
    } else {
        // Best-effort attempt at translating return codes
        *__errno() = match ctru_sys::R_SUMMARY(ret) as libc::c_uint {
            ctru_sys::RS_WOULDBLOCK => libc::EAGAIN,
            ctru_sys::RS_INVALIDARG | ctru_sys::RS_WRONGARG => {
                match ctru_sys::R_DESCRIPTION(ret) as libc::c_uint {
                    // The handle is incorrect (even though we just made it)
                    ctru_sys::RD_INVALID_HANDLE => ECTRU,
                    _ => libc::EINVAL,
                }
            }
            _ => ECTRU,
        };

        -1
    }
}
*/
