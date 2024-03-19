use bevy::{
    app::Plugin,
    asset::{Assets, Handle},
    render::{
        render_asset::PrepareAssetError,
        render_resource::Extent3d,
        texture::{Image, TextureFormatPixelInfo},
        RenderApp,
    },
};
use citro3d::texture::{Tex, TexFormat, TexParams};
use log::{trace, warn};
use swizzle_3ds::pix::ImageView;

use crate::gpu_buffer::LinearBuffer;

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

pub const BLANK_TEXTURE: Handle<Image> = Handle::weak_from_u128(0x48cefbd5e0f04f7b85a79f5735bd49fc);

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

        let mut assets = app.world.resource_mut::<Assets<Image>>();
        assets.insert(
            BLANK_TEXTURE,
            Image::from_dynamic(
                image::RgbaImage::from_fn(8, 8, |_, _| image::Rgba([255u8, 255, 255, 255])).into(),
                true,
            ),
        );
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
    fn from_bevy(mut img: Image) -> Option<Self> {
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

        use swizzle_3ds::pix as spix;
        let img_format = match img.texture_descriptor.format {
            bevy::render::render_resource::TextureFormat::R8Unorm
            | bevy::render::render_resource::TextureFormat::R8Uint => spix::ImageFormat::Lum8,
            bevy::render::render_resource::TextureFormat::Rg8Unorm => spix::ImageFormat::Luma8,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm
            | bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb => {
                spix::ImageFormat::Rgba8
            }
            _ => return None,
        };

        if img.width() < 8 || img.height() < 8 {
            img.resize(Extent3d {
                width: img.width().max(8),
                height: img.height().max(8),
                ..Default::default()
            });
        }

        let tex_buf = LinearBuffer::with_size(
            img.width() as usize
                * img.height() as usize
                * img.texture_descriptor.format.pixel_size(),
            0u8,
        );

        let img_view = ImageView::new(
            img.data.as_ref(),
            img.width() as usize,
            img.height() as usize,
            img_format,
        );
        let swiz_img = swizzle_3ds::swizzle_image_into(&img_view, tex_buf);

        let tex = Tex::new(
            TexParams::new_2d(
                img.width().try_into().expect("image too wide"),
                img.height().try_into().expect("image too tall"),
            )
            .format(match img_format {
                spix::ImageFormat::Rgba8 => TexFormat::Rgba8,
                spix::ImageFormat::Rgb8 => TexFormat::Rgb8,
                spix::ImageFormat::Rgba5551 => TexFormat::Rgba5551,
                spix::ImageFormat::Rgb565 => TexFormat::Rgb565,
                spix::ImageFormat::Rgba4 => TexFormat::Rgba4,
                spix::ImageFormat::Luma8 => TexFormat::La8,
                spix::ImageFormat::Hilo8 => TexFormat::HiLo8,
                spix::ImageFormat::Lum8 => TexFormat::L8,
                spix::ImageFormat::Alpha8 => TexFormat::A8,
                spix::ImageFormat::Luma4 => TexFormat::La4,
                _ => unimplemented!("unhandled texture format"),
            })
            .use_vram(false),
        )
        .ok()?;
        let bytes = swiz_img.as_raw();

        tex.upload(bytes);
        Some(Self(tex))
    }

    pub fn width(&self) -> f32 {
        self.0.width() as f32
    }
    pub fn height(&self) -> f32 {
        self.0.height() as f32
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn data(&self) -> &[u8] {
        self.0.data()
    }
}

impl PrepareAsset for Image {
    type PreparedAsset = GpuImage;
    type Param = ();

    fn prepare_asset_3ds(
        extracted: <Self as bevy::render::render_asset::RenderAsset>::ExtractedAsset,
        _: &mut bevy::ecs::system::SystemParamItem<<Self as PrepareAsset>::Param>,
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
