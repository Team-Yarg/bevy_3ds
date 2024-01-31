use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    hierarchy::HierarchyPlugin,
    text::TextPlugin,
    transform::TransformPlugin,
    window::{Window, WindowPlugin, WindowResolution},
};
use bevy_3ds_input::InputPlugin;
use bevy_3ds_pbr::Bevy3dsPbrPlugin;
use bevy_3ds_render::texture::ImagePlugin;

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
            .add(InputPlugin)
            //.add_plugins(romfs_assets::RomfsAssetPlugin)
            .add(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(400., 240.),
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
        group = group.add(Bevy3dsPbrPlugin);
        group = group.add(render::plugin::CorePipeline3ds);
        group = group.add(sprite::SpritesPlugin).add(TextPlugin);
        // group = group.add(UiPlugin::default());
        group = group.add(PbrPlugin::default());
        group
    }
}
