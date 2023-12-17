use std::io::Write;
use std::{fs::File, panic::PanicInfo};

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
use ctru::services::{
    self,
    apt::Apt,
    gfx::Gfx,
    hid::{Hid, KeyPad},
};

mod shims;

//use libc::c_void;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let mut f = File::create("panics.log").unwrap();
        write!(f, "{}", info).ok();
    }));
    let mut hid = Hid::new().unwrap();
    let apt = Apt::new().unwrap();
    let gfx = Gfx::new().unwrap();
    /*let mut buf = [0u8; 32];
    unsafe {
        //libc::open("".as_ptr(), 0);
        libc::getrandom(buf.as_mut_ptr() as *mut c_void, buf.len(), 0);
    }*/
    let tty = ctru::console::Console::new(gfx.bottom_screen.borrow_mut());

    let mut app = App::new();
    app.set_runner(move |mut app| {
        while apt.main_loop() {
            //gfx.wait_for_vblank();
            app.update();
        }
    });
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, setup);
    println!("hello");
    app.run();
}

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
