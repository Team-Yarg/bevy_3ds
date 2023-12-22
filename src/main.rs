use std::alloc::Layout;
use std::io::Write;
use std::{fs::File, panic::PanicInfo};

use bevy::ecs::schedule::{Schedule, ScheduleGraph};
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
mod shims;

#[cfg(target_os = "horizon")]
mod plugin_3ds;

//use libc::c_void;

#[cfg(target_os = "horizon")]
fn ds_main() {
    use std::{cell::Cell, hash::RandomState};

    use bevy::{
        asset::AssetPlugin, core_pipeline::CorePipelinePlugin, hierarchy::HierarchyPlugin,
        render::texture::ImagePlugin, sprite::SpritePlugin, text::TextPlugin,
        transform::TransformPlugin, ui::UiPlugin, MinimalPlugins,
    };
    use indexmap::IndexMap;

    use crate::plugin_3ds::Render3dsPlugin;

    let gfx = Gfx::new().unwrap();
    let tty = ctru::console::Console::new(gfx.bottom_screen.borrow_mut());

    std::env::set_var("BEVY_ASSET_ROOT", "romfs://");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((TransformPlugin, HierarchyPlugin, bevy::input::InputPlugin))
        .add_plugins(bevy::asset::AssetPlugin {
            file_path: "romfs://".to_owned(),
            processed_file_path: "res://".to_owned(),
            watch_for_changes_override: None,
            mode: bevy::asset::AssetMode::Unprocessed,
        })
        .add_plugins((SpritePlugin, TextPlugin, CorePipelinePlugin));
    app.add_plugins(Render3dsPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ImagePlugin::default());
    app.add_systems(Startup, setup);

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

fn setup(mut cmds: Commands) {
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
