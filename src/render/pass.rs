use std::{error::Error, marker::PhantomData, sync::Arc};

use bevy::{
    asset::Handle,
    ecs::{
        query::{QueryItem, ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
        system::{SystemParam, SystemParamItem},
    },
};
use citro3d::{buffer::Primitive, render::Target, shader::Program, uniform::Index};

use crate::gpu_buffer::LinearBuffer;

use super::{
    pipeline::{RenderPipelineDescriptor, ShaderLib, VertexAttrs},
    shader::PicaShader,
    GpuDevice, VboSlice,
};
type Result<T, E = RenderError> = std::result::Result<T, E>;

pub struct RenderPass<'g> {
    gpu: &'g GpuDevice,
}
impl<'g> RenderPass<'g> {
    pub fn new(gpu: &'g GpuDevice, target: &Target) -> citro3d::Result<Self> {
        unsafe {
            citro3d_sys::C3D_FrameBegin(citro3d_sys::C3D_FRAME_SYNCDRAW.try_into().unwrap());
        }
        gpu.instance.lock().unwrap().select_render_target(target)?;
        Ok(Self { gpu })
    }

    fn set_vertex_shader<'f>(
        &'f mut self,
        shader: &'f PicaShader,
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

    pub fn set_pipeline<'frame>(
        &'frame mut self,
        pl: RenderPipelineDescriptor<'frame>,
    ) -> Result<()> {
        let mut act = move || {
            self.set_vertex_shader(pl.vertex.shader, pl.vertex.entry_point)?;
            self.gpu.set_attr_info(&pl.vertex.attrs);
            Ok(())
        };
        act().map_err(|e| RenderError::PipelineError {
            label: pl.label,
            error: e,
        })
    }
    pub fn add_vertex_buffer<'f, T>(&'f mut self, verts: &'f LinearBuffer<T>) -> Result<VboSlice> {
        let slice = unsafe { self.gpu.add_vertex_buffer(verts)? };
        Ok(slice)
    }
    pub fn bind_vertex_uniform(&mut self, index: Index, uni: impl citro3d::uniform::Uniform) {
        self.gpu
            .instance
            .lock()
            .unwrap()
            .bind_vertex_uniform(index, uni);
    }
    pub fn draw<'f>(&mut self, verts: &VboSlice, prim: Primitive) {
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
