use crate::{frame::Citro3dFrame, gpu_buffer::LinearBuffer};

use super::{pipeline::VertexAttrs, shader::PicaShader, GpuDevice, GpuImage};
use bevy::ecs::{
    entity::Entity,
    system::{SystemParam, SystemParamItem},
};
use citro3d::{
    buffer::Primitive,
    render::Target,
    shader::{self, Program},
    uniform::Index,
};
use std::{marker::PhantomData, sync::Arc};
type Result<T, E = RenderError> = std::result::Result<T, E>;

pub struct VboSlice<'vbo, 'buf> {
    _buf: PhantomData<&'buf VboBuffer>,
    slice: citro3d::buffer::Slice<'vbo>,
}
/// Wrapper for [`citro3d::buffer::Info`] that allows better lifetime management
pub struct VboBuffer {
    buf: citro3d::buffer::Info,
}
impl VboBuffer {
    pub fn new() -> Self {
        Self {
            buf: citro3d::buffer::Info::new(),
        }
    }
    pub fn add<'vbo, 'me, T>(
        &'me mut self,
        vbo: &'vbo LinearBuffer<T>,
        attrib_info: &citro3d::attrib::Info,
    ) -> citro3d::Result<VboSlice<'vbo, 'me>> {
        let slice = self.buf.add(vbo, attrib_info)?;
        let slice = unsafe { std::mem::transmute::<_, citro3d::buffer::Slice<'vbo>>(slice) };
        Ok(VboSlice {
            slice,
            _buf: PhantomData,
        })
    }
}

impl Default for VboBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RenderPass<'g, 'f> {
    gpu: &'g GpuDevice,
    _frame: &'f Citro3dFrame<'g>,
}
impl<'g, 'f> RenderPass<'g, 'f> {
    pub fn new(gpu: &'g GpuDevice, _frame: &'f Citro3dFrame<'g>) -> Self {
        Self { gpu, _frame }
    }
    pub fn select_render_target(&mut self, target: &Target) {
        self.gpu
            .inst()
            .select_render_target(target)
            .expect("failed to set render target even though we are in a frame, thats unexpected");
    }

    pub fn set_vertex_shader(&mut self, shader: &'f PicaShader, entry_point: usize) -> Result<()> {
        let prog = Arc::pin(Program::new(
            shader
                .entry_point(entry_point)
                .ok_or(RenderError::InvalidEntryPoint { index: entry_point })?,
        )?);
        // Safety: we put the program in our store after this
        unsafe {
            self.gpu.set_shader(prog);
        };
        Ok(())
    }
    pub fn set_attr_info(&self, info: &VertexAttrs) {
        self.gpu.set_attr_info(info);
    }
    /// Configure a texenv stage
    pub fn configure_texenv<R>(
        &self,
        stage: citro3d::texenv::Stage,
        f: impl FnOnce(&mut citro3d::texenv::TexEnv) -> R,
    ) -> R {
        let mut gpu = self.gpu.inst();
        let env = gpu.texenv(stage);
        f(env)
    }
    pub fn bind_texture(&mut self, index: i32, tex: &'f GpuImage) {
        tex.0.bind(index);
    }

    pub fn bind_vertex_uniform(&mut self, index: Index, uni: impl citro3d::uniform::Uniform) {
        self.gpu
            .instance
            .lock()
            .unwrap()
            .bind_vertex_uniform(index, uni);
    }
    pub fn bind_vertex_uniform_bevy(&mut self, index: Index, mat: &bevy::math::Mat4) {
        let _gpu = self.gpu.inst();
        let mut cells = mat.transpose().to_cols_array();
        cells[0..4].reverse();
        cells[4..8].reverse();
        cells[8..12].reverse();
        cells[12..16].reverse();
        let c3d_mat = citro3d_sys::C3D_Mtx { m: cells };

        // Safety: It actually does a deep copy of the matrix so we arn't leaving a pointer dangling
        unsafe {
            citro3d_sys::C3D_FVUnifMtxNx4(shader::Type::Vertex.into(), index.into(), &c3d_mat, 4);
        }
    }

    pub fn draw(&mut self, prim: Primitive, verts: VboSlice<'f, '_>) {
        unsafe {
            self.gpu.draw(prim, verts.slice);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("generic render issue")]
    Generic,
    #[error("pipeline error {label:#?} {error}")]
    PipelineError {
        label: Option<&'static str>,
        error: Box<RenderError>,
    },
    #[error("invalid shader entry point: {index}")]
    InvalidEntryPoint { index: usize },
    #[error("internal ctru {0}")]
    Ctru(#[from] ctru::Error),
    #[error("internal citro3d {0}")]
    Citro3d(#[from] citro3d::Error),
}

pub trait RenderCommand {
    type Param: SystemParam + 'static;

    fn render<'w: 'f, 'f>(
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut RenderPass<'w, 'f>,
        view: Entity,
    ) -> Result<(), RenderError>;
}
