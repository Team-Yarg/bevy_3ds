use bevy::{
    app::Plugin,
    render::{
        render_asset::PrepareAssetError, render_resource::TextureDimension, texture::Image,
        RenderApp,
    },
};
use citro3d::texture::{Tex, TexParams};
use image::EncodableLayout;
use log::{trace, warn};

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

#[derive(Default)]
pub struct ImagePlugin {
    /// we proxy stuff to this but intercept calls to functions which try and reference stuff we don't support
    /// e.g. RenderDevice
    inner: bevy::render::texture::ImagePlugin,
}

impl Plugin for ImagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        self.inner.build(app);
        app.add_plugins(PrepareAssetsPlugin::<Image>::default());
    }

    fn ready(&self, app: &bevy::prelude::App) -> bool {
        self.inner.ready(app)
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        // this is an _awful_ hack to get as much of the `finish` functionality as possible
        // but avoid the render-app specific stuff because it needs RenderDevice
        if let Some(render_app) = app.remove_sub_app(RenderApp) {
            // todo: put fallback images here
            self.inner.finish(app);
            app.insert_sub_app(RenderApp, render_app);
        } else {
            self.inner.finish(app);
        }
    }

    fn cleanup(&self, app: &mut bevy::prelude::App) {
        self.inner.cleanup(app);
    }

    fn name(&self) -> &str {
        // prevent being loaded along with the normal one
        std::any::type_name::<ImagePlugin>()
    }

    fn is_unique(&self) -> bool {
        true
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
        let img = img.to_rgba8();
        let mut bytes = img.as_bytes().to_owned();
        for px in bytes.chunks_mut(4) {
            px.reverse();
        }
        tex.upload(&bytes);
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
