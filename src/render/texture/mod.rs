use bevy::{app::Plugin, render::texture::Image};

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PrepareAssetsPlugin::<Image>::default());
    }
}

pub struct GpuImage {}

impl PrepareAsset for Image {
    type PreparedAsset = GpuImage;
    type Param = ();

    fn prepare_asset_3ds(
        extracted: <Self as bevy::render::render_asset::RenderAsset>::ExtractedAsset,
        param: &mut bevy::ecs::system::SystemParamItem<<Self as PrepareAsset>::Param>,
    ) -> Result<
        <Self as PrepareAsset>::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<
            <Self as bevy::render::render_asset::RenderAsset>::ExtractedAsset,
        >,
    > {
        Ok(GpuImage {})
    }
}
