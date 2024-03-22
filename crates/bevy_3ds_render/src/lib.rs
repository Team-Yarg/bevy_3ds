#![feature(allocator_api)]

use self::pipeline::VertexAttrs;
use bevy::{
    asset::Handle,
    ecs::{component::Component, schedule::SystemSet, system::Resource},
    render::{
        extract_component::ExtractComponent, render_resource::PrimitiveTopology, texture::Image,
    },
};
use citro3d::{buffer::Primitive, Instance};
use frame::Citro3dFrame;
pub use plugin::Render3dsPlugin;
pub use prep_asset::RenderAssets;
use std::{
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
};

pub use texture::GpuImage;
mod bottom_screen;
pub mod draw;
mod extract;
mod frame;
pub mod gpu_buffer;
pub mod lighting;
pub mod material;
pub mod materials;
pub mod mesh;
pub mod pass;
pub mod pipeline;
pub mod plugin;
mod prep_asset;
pub mod shader;
pub mod texture;
pub mod vertattr;

pub use citro3d;

pub struct GfxInstance(pub ctru::services::gfx::Gfx);

#[derive(Component, Clone, Copy, Default, ExtractComponent)]
#[non_exhaustive]
pub enum On3dsScreen {
    #[default]
    Top,
    Bottom,
}

impl Default for GfxInstance {
    fn default() -> Self {
        Self(
            ctru::services::gfx::Gfx::with_formats_shared(
                ctru::services::gspgpu::FramebufferFormat::Rgba8,
                ctru::services::gspgpu::FramebufferFormat::Rgba8,
            )
            .expect("failed to init gpu"),
        )
    }
}

#[derive(Resource, Default, Clone)]
pub struct BottomScreenTexture(Handle<Image>);
impl BottomScreenTexture {
    pub fn new(value: Handle<Image>) -> Self {
        Self(value)
    }
}

#[derive(Resource)]
pub struct GpuDevice {
    instance: Mutex<citro3d::Instance>,
}
impl Default for GpuDevice {
    fn default() -> Self {
        let instance = Mutex::new(citro3d::Instance::new().unwrap());
        Self { instance }
    }
}
impl GpuDevice {
    fn inst(&self) -> MutexGuard<Instance> {
        self.instance.lock().unwrap()
    }
    pub fn start_new_frame(&self) -> Citro3dFrame<'_> {
        Citro3dFrame::new(self)
    }

    /// set the shader program to use for subsequent draw calls
    ///
    /// # Safety
    /// If the shader is drop'd before the frame ends then it will result in a use-after-free
    pub unsafe fn set_shader(&self, shader: Pin<Arc<citro3d::shader::Program>>) {
        self.inst().bind_program(shader);
    }

    /// Set the attribute info for subsequent draw calls
    pub fn set_attr_info(&self, attr: &VertexAttrs) {
        self.inst().set_attr_info(&attr.0);
    }

    /// Draw vertexes
    ///
    /// # Safety
    /// If `verts` goes out of scope before the frame ends there will be a use-after-free by the GPU
    pub unsafe fn draw(&self, prim: Primitive, verts: citro3d::buffer::Slice) {
        self.inst().draw_arrays(prim, verts);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum RenderSet3ds {
    PrepareAssets,
    Prepare,
    PrepareBindGroups,
}

pub fn bevy_topology_to_citro(topology: PrimitiveTopology) -> Option<Primitive> {
    match topology {
        PrimitiveTopology::TriangleList => Some(Primitive::Triangles),
        PrimitiveTopology::TriangleStrip => Some(Primitive::TriangleStrip),
        _ => None,
    }
}
