#![feature(allocator_api)]
use bevy::{
    input::Input,
    app::{App, Plugin, PluginsState},
    tasks::tick_global_task_pools_on_main_thread,
};

pub mod render {
    pub use bevy_3ds_render::*;
}

pub mod sprite {
    pub use bevy_3ds_sprite::*;
}

mod default_plugins;

use bevy_3ds_input::button::{_3dsButton, _3dsButtonType};
use ctru::prelude::*;
pub use default_plugins::DefaultPlugins;

pub struct Core3dsPlugin;

impl Plugin for Core3dsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        std::env::set_var("BEVY_ASSET_ROOT", "romfs:/");
        app.set_runner(app_runner);
    }
}

fn app_runner(
    mut app: App
) {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let apt = Apt::new().unwrap();
    while apt.main_loop() {
        let buttons = app.world.get_resource::<Input<_3dsButton>>().expect("Input<_3dsButton> resource not found");

        if buttons.pressed(_3dsButton::new(_3dsButtonType::Start)) {
            break;
        }

        if app.plugins_state() != PluginsState::Cleaned {
            if app.plugins_state() != PluginsState::Ready {
                tick_global_task_pools_on_main_thread();
            } else {
                app.finish();
                app.cleanup();
            }
        }

        if app.plugins_state() == PluginsState::Cleaned {
            app.update();
        }
    }
}
