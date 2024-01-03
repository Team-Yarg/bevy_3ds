use std::{
    mem::MaybeUninit,
    ops::Range,
    pin::Pin,
    sync::{Arc, LockResult, Mutex, MutexGuard},
};

use bevy::ecs::{schedule::SystemSet, system::Resource};

pub mod draw;
mod extract;
mod mesh;
pub mod pass;
pub mod pipeline;
mod plugin;
mod prep_asset;
pub mod shader;
mod texture;

use citro3d::{
    attrib,
    buffer::{self, Primitive},
    Instance,
};
pub use plugin::Render3dsPlugin;
pub use prep_asset::RenderAssets;

use crate::gpu_buffer::LinearBuffer;

use self::pipeline::{ShaderLib, VertexAttrs};

struct BufferState(citro3d_sys::C3D_BufInfo);

impl BufferState {
    fn new() -> Self {
        // Safety: BufInit_Init will initalise the data
        let info = unsafe {
            let mut info = MaybeUninit::zeroed();
            citro3d_sys::BufInfo_Init(info.as_mut_ptr());
            info.assume_init()
        };
        Self(info)
    }
    fn add<T>(
        &mut self,
        verts: &[T],
        attrib_info: &attrib::Info,
    ) -> Result<VboSlice, citro3d::Error> {
        let stride = std::mem::size_of::<T>().try_into()?;

        let res = unsafe {
            citro3d_sys::BufInfo_Add(
                &mut self.0,
                verts.as_ptr().cast(),
                stride,
                attrib_info.attr_count(),
                attrib_info.permutation(),
            )
        };

        // Error codes from <https://github.com/devkitPro/citro3d/blob/master/source/buffers.c#L11>
        match res {
            ..=-3 => Err(citro3d::Error::System(res)),
            -2 => Err(citro3d::Error::InvalidMemoryLocation),
            -1 => Err(citro3d::Error::TooManyBuffers),
            _ => Ok(VboSlice {
                index: res as usize,
                range: 0..verts.len(),
            }),
        }
    }
}

impl Default for BufferState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VboBufIndex {}
#[derive(Debug)]
pub struct VboSlice {
    index: usize,
    range: Range<usize>,
}

pub struct GfxInstance(ctru::services::gfx::Gfx);
impl Default for GfxInstance {
    fn default() -> Self {
        Self(ctru::services::gfx::Gfx::new().expect("failed to init gpu"))
    }
}

#[derive(Resource)]
pub struct GpuDevice {
    instance: Mutex<citro3d::Instance>,
}
impl Default for GpuDevice {
    fn default() -> Self {
        let mut buf_info = BufferState::default();
        let instance = Mutex::new(citro3d::Instance::new().unwrap());
        unsafe {
            citro3d_sys::C3D_SetBufInfo(&mut buf_info.0);
        };
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
