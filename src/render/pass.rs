use std::{error::Error, marker::PhantomData, sync::Arc};

use bevy::{
    asset::Handle,
    ecs::{
        query::{QueryItem, ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
        system::{SystemParam, SystemParamItem},
    },
};
use citro3d::{
    buffer::Primitive,
    render::Target,
    shader::{self, Program},
    uniform::Index,
};

use crate::gpu_buffer::LinearBuffer;

use super::{
    pipeline::{RenderPipelineDescriptor, ShaderLib, VertexAttrs},
    shader::PicaShader,
    GpuDevice, GpuImage, VboSlice,
};
type Result<T, E = RenderError> = std::result::Result<T, E>;

pub struct RenderPass<'g> {
    gpu: &'g GpuDevice,
}
impl<'g> RenderPass<'g> {
    pub fn new(gpu: &'g GpuDevice) -> citro3d::Result<Self> {
        unsafe {
            citro3d_sys::C3D_FrameBegin(citro3d_sys::C3D_FRAME_SYNCDRAW.try_into().unwrap());
        }
        Ok(Self { gpu })
    }
    pub fn select_render_target(&mut self, target: &Target) {
        self.gpu
            .inst()
            .select_render_target(target)
            .expect("failed to set render target even though we are in a frame, thats unexpected");
    }

    pub fn set_vertex_shader<'f>(
        &'f mut self,
        shader: &'g PicaShader,
        entry_point: usize,
    ) -> Result<()> {
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
    pub fn bind_texture(&self, index: i32, tex: &'g GpuImage) {
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

    pub fn draw<'f>(&mut self, prim: Primitive, verts: citro3d::buffer::Slice) {
        unsafe {
            self.gpu.draw(prim, verts);
        }
    }
}
impl<'g> Drop for RenderPass<'g> {
    fn drop(&mut self) {
        unsafe {
            citro3d_sys::C3D_FrameEnd(0);
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

    fn render<'w, 'f, 'g>(
        param: SystemParamItem<'w, 'f, Self::Param>,
        pass: &'f mut RenderPass<'g>,
    ) -> Result<(), RenderError>;
}
