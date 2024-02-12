use bevy::{
    app::Plugin,
    asset::{AssetApp, AssetId, Assets, Handle},
    pbr::StandardMaterial,
    render::{color::Color, extract_instances::ExtractInstancesPlugin, ExtractSchedule, RenderApp},
};
use lights::extract_point_lights;

mod lights;

pub struct Bevy3dsPbrPlugin;

impl Plugin for Bevy3dsPbrPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_reflect::<StandardMaterial>();
        app.init_asset::<StandardMaterial>();
        app.add_plugins(ExtractInstancesPlugin::<AssetId<StandardMaterial>>::extract_visible());
        app.world.resource_mut::<Assets<StandardMaterial>>().insert(
            Handle::<StandardMaterial>::default(),
            StandardMaterial {
                base_color: Color::rgb(1.0, 0.0, 0.5),
                unlit: true,
                ..Default::default()
            },
        );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, extract_point_lights);
        }
    }
}
