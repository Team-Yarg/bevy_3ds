#![feature(allocator_api)]

use std::{
    mem::MaybeUninit,
    ops::Range,
    pin::Pin,
    sync::{Arc, LockResult, Mutex, MutexGuard},
};

use bevy::ecs::{schedule::SystemSet, system::Resource};

pub mod draw;
mod extract;
pub mod gpu_buffer;
mod mesh;
pub mod pass;
pub mod pipeline;
pub mod plugin;
mod prep_asset;
pub mod shader;
pub mod texture;

use citro3d::{
    attrib,
    buffer::{self, Primitive},
    Instance,
};
pub use plugin::Render3dsPlugin;
pub use prep_asset::RenderAssets;
pub use texture::GpuImage;

use crate::gpu_buffer::LinearBuffer;

use self::pipeline::{ShaderLib, VertexAttrs};

pub struct GfxInstance(ctru::services::gfx::Gfx);
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
