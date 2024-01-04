use bevy::{
    app::Plugin,
    asset::{
        io::{AssetSource, AssetSourceBuilder, AssetSourceId},
        AssetApp,
    },
};

use self::reader::RomfsAssetReader;

mod reader;

pub struct RomfsAssetPlugin;

impl Plugin for RomfsAssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(|| Box::new(RomfsAssetReader)),
        );
    }
}
