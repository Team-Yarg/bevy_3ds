use std::ops::Deref;

use bevy::{
    app::Plugin,
    asset::{Assets, Handle},
    ecs::{
        schedule::IntoSystemConfigs,
        system::{Commands, NonSend, Res, ResMut, Resource},
    },
    render::{texture::Image, Extract, ExtractSchedule, Render, RenderApp, RenderSet},
};
use ctru::services::{
    gfx::{Flush, RawFrameBuffer, Screen, Swap},
    gspgpu::FramebufferFormat,
};

use crate::{BottomScreenTexture, GfxInstance, RenderAssets};

pub struct BottomScreenPlugin;

impl Plugin for BottomScreenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(Render, render_bottom_screen.in_set(RenderSet::Render))
            .add_systems(ExtractSchedule, extract_draw_bottom_screen)
            .init_resource::<ExtractedBottomScreenTexture>();
    }
}

#[derive(Default, Resource)]
struct ExtractedBottomScreenTexture {
    texture: Option<Handle<Image>>,
    format: Option<FramebufferFormat>,
}

fn render_bottom_screen(
    texture: Res<ExtractedBottomScreenTexture>,
    textures: Res<RenderAssets<Image>>,
    gfx: NonSend<GfxInstance>,
) {
    let Some(format) = texture.format else {
        return;
    };
    let tex = texture.texture.as_ref().unwrap();
    let Some(tex) = textures.get(tex) else {
        return;
    };

    let mut bottom_screen = gfx.0.bottom_screen.borrow_mut();
    bottom_screen.set_framebuffer_format(format);

    let RawFrameBuffer { ptr, .. } = bottom_screen.raw_framebuffer();
    unsafe { ptr.copy_from(tex.data().as_ptr(), tex.data().len()) };

    bottom_screen.flush_buffers();
    bottom_screen.swap_buffers();
}

fn extract_draw_bottom_screen(
    mut extracted: ResMut<ExtractedBottomScreenTexture>,
    bot: Extract<Option<Res<BottomScreenTexture>>>,
    assets: Extract<Res<Assets<Image>>>,
) {
    let Some(tex) = bot.deref() else {
        extracted.format.take();
        return;
    };
    extracted.texture.replace(tex.0.clone());
    extracted.format.replace(FramebufferFormat::Rgba8);
}
