#![feature(allocator_api)]
use bevy::{
    app::Plugin,
    hierarchy::HierarchyPlugin,
    render::texture::ImagePlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    transform::TransformPlugin,
    ui::UiPlugin,
    window::{Window, WindowPlugin, WindowResolution},
    MinimalPlugins,
};

pub mod render {
    pub use bevy_3ds_render::*;
}

pub mod sprite {
    pub use bevy_3ds_sprite::*;
}

mod default_plugins;

pub use default_plugins::DefaultPlugins;

pub struct Core3dsPlugin;

impl Plugin for Core3dsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        std::env::set_var("BEVY_ASSET_ROOT", "romfs:/");
    }
}
