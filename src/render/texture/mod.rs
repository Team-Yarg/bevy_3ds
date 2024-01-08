use bevy::{
    app::Plugin,
    render::{render_asset::PrepareAssetError, render_resource::TextureDimension, texture::Image},
};
use citro3d::texture::{Tex, TexParams};
use image::EncodableLayout;
use log::{trace, warn};

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PrepareAssetsPlugin::<Image>::default());
    }
}

#[derive(Debug)]
pub struct GpuImage(pub(super) citro3d::texture::Tex);

const MAX_TEX_SIZE: u32 = 1024;

impl GpuImage {
    fn from_bevy(img: Image) -> Option<Self> {
        let desc = &img.texture_descriptor;
        assert!(
            img.width() <= MAX_TEX_SIZE,
            "image is too wide, max is {}",
            MAX_TEX_SIZE
        );
        assert!(
            img.height() <= MAX_TEX_SIZE,
            "image is too tall, max is {}",
            MAX_TEX_SIZE
        );
        let mut img = img.try_into_dynamic().ok()?;
        if img.width() < 8 || img.height() < 8 {
            img = img.resize_exact(
                img.width().max(8),
                img.height().max(8),
                image::imageops::FilterType::Nearest,
            );
        }
        let img = swizzle_3ds::swizzle_image(&img);

        let mut tex = Tex::new(
            TexParams::new_2d(
                img.width().try_into().expect("image too wide"),
                img.height().try_into().expect("image too tall"),
            )
            .format(citro3d::texture::TexFormat::Rgba8),
        )
        .ok()?;
        tex.upload(img.to_rgba8().as_bytes());
        Some(Self(tex))
    }

    pub fn width(&self) -> f32 {
        self.0.width() as f32
    }
    pub fn height(&self) -> f32 {
        self.0.height() as f32
    }
}

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
        trace!(
            "prepare image for 3ds gpu {:#?}",
            extracted.texture_descriptor.label
        );
        match GpuImage::from_bevy(extracted.clone()) {
            Some(i) => Ok(i),
            None => {
                warn!("failed to load image");
                Err(PrepareAssetError::RetryNextUpdate(extracted))
            }
        }
    }
}
