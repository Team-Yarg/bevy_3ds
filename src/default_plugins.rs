use super::Core3dsPlugin;
use crate::plugins;

#[cfg(feature = "render")]
use crate::render;

#[cfg(feature = "sprite")]
use crate::sprite;

use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    hierarchy::HierarchyPlugin,
    scene::ScenePlugin,
    transform::TransformPlugin,
    window::{Window, WindowPlugin, WindowResolution},
};
use bevy_3ds_input::InputPlugin;
#[cfg(feature = "pbr")]
use bevy_3ds_pbr::Bevy3dsPbrPlugin;
#[cfg(feature = "render")]
use bevy_3ds_render::texture::ImagePlugin;

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
        #[cfg(feature = "render")]
        {
            group = group
                .add(render::plugin::Render3dsPlugin)
                .add(ImagePlugin::default());
        }
        #[cfg(feature = "pbr")]
        {
            group = group.add(Bevy3dsPbrPlugin);
        }
        group = group.add(plugins::CorePipeline3ds);
        #[cfg(feature = "sprite")]
        {
            group = group.add(sprite::SpritesPlugin).add(bevy::text::TextPlugin);
        }
        //group = group.add(UiPlugin::default());
        group = group.add(ScenePlugin);
        #[cfg(feature = "gltf")]
        {
            group = group.add(bevy::gltf::GltfPlugin::default());
        }
        group
    }
}
