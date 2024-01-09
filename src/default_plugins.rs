use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    hierarchy::HierarchyPlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    transform::TransformPlugin,
    window::{Window, WindowPlugin, WindowResolution},
    MinimalPlugins,
};
use bevy_3ds_render::texture::ImagePlugin;
use bevy_3ds_ui::UiPlugin;

use super::Core3dsPlugin;
use crate::{render, sprite};

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();

        group = group
            .add(bevy::core::TaskPoolPlugin::default())
            .add(bevy::core::TypeRegistrationPlugin)
            .add(bevy::core::FrameCountPlugin)
            .add(bevy::time::TimePlugin)
            .add(bevy::app::ScheduleRunnerPlugin::default())
            .add(Core3dsPlugin)
            .add(TransformPlugin)
            .add(HierarchyPlugin)
            .add(bevy::input::InputPlugin)
            //.add_plugins(romfs_assets::RomfsAssetPlugin)
            .add(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(480., 240.),
                    resizable: false,
                    ..Default::default()
                }),
                ..Default::default()
            });
        group = group.add(bevy::asset::AssetPlugin::default());
        {
            group = group
                .add(render::plugin::Render3dsPlugin)
                .add(ImagePlugin::default());
        }
        group = group.add(render::plugin::CorePipeline3ds);
        group = group.add(sprite::SpritesPlugin).add(TextPlugin);
        group = group.add(UiPlugin::default());
        group
    }
}