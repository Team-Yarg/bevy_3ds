use std::{error::Error, marker::PhantomData};

use bevy::ecs::{
    query::ROQueryItem,
    system::{SystemParam, SystemParamItem},
};
use citro3d::{buffer::Primitive, shader::Program};

use super::{
    pipeline::{RenderPipelineDescriptor, ShaderLib, VertexAttrs},
    GpuDevice, VboSlice,
};
type Result<T, E = RenderError> = std::result::Result<T, E>;

pub struct RenderPass<'g> {
    gpu: &'g GpuDevice,
}
impl<'g> RenderPass<'g> {
    pub fn new(gpu: &'g GpuDevice) -> Self {
        Self { gpu }
    }

    fn set_vertex_shader<'f>(
        &'f mut self,
        shader: &'f ShaderLib,
        entry_point: usize,
    ) -> Result<()> {
        let prog = Program::new(
            shader
                .get(entry_point)
                .ok_or(RenderError::InvalidEntryPoint(entry_point))?,
        )
        .map_err(|e| e.into())?;
        // Safety: The lifetime bounds on this method prevent it going out of frame
        unsafe {
            self.gpu.set_shader(&prog);
        };
        Ok(())
    }

    pub fn set_pipeline<'frame>(
        &'frame mut self,
        pl: RenderPipelineDescriptor<'frame>,
    ) -> Result<()> {
        let mut act = move || {
            self.set_vertex_shader(pl.vertex.shader, pl.vertex.entry_point)?;
            Ok(())
        };
        act().map_err(|e| RenderError::PipelineError {
            label: pl.label,
            error: e,
        })
    }
    pub fn add_vertex_buffer<'f, T>(
        &'f mut self,
        verts: &'f [T],
        attrs: &'f VertexAttrs,
    ) -> Result<VboSlice> {
        unsafe {
            self.gpu.add_vertex_buffer(verts, attrs)?;
        }
        Ok(())
    }
    pub fn draw<'f>(&mut self, verts: &VboSlice, prim: Primitive) {
        unsafe {
            self.gpu.draw(prim, verts);
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
    type Param: SystemParam;
    type ItemData: ReadOnlyQueryData;

    fn render<'w>(
        entity: ROQueryItem<'w, Self::ItemData>,
        param: &SystemParamItem<'w, '_, Self::Param>,
        pass: &mut RenderPass,
    ) -> Result<(), RenderError>;
}
