use crate::{frame::Citro3dFrame, gpu_buffer::LinearBuffer, material::Material};

use super::{pipeline::VertexAttrs, shader::PicaShader, GpuDevice, GpuImage};
use bevy::{
    ecs::{
        entity::Entity,
        system::{SystemParam, SystemParamItem},
    },
    render::color::Color,
};
use citro3d::{buffer::Primitive, render::Target, shader::Program, uniform::Index};
use std::{marker::PhantomData, ops::Deref, sync::Arc};
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

    pub fn bind_vertex_uniform(&mut self, index: Index, uni: impl Into<citro3d::uniform::Uniform>) {
        self.gpu
            .instance
            .lock()
            .unwrap()
            .bind_vertex_uniform(index, uni);
    }
    pub fn set_lighting_material(&mut self, mat: Material) {
        let citro_mat = citro3d::material::Material {
            ambient: mat.ambient.map(bevy_to_citro3d_clr),
            diffuse: mat.diffuse.map(bevy_to_citro3d_clr),
            specular0: mat.specular0.map(bevy_to_citro3d_clr),
            specular1: mat.specular1.map(bevy_to_citro3d_clr),
            emission: mat.emission.map(bevy_to_citro3d_clr),
        };
        let mut gpu = self.gpu.inst();
        let mut light_env = gpu.light_env_mut();
        for (i, u, d) in mat.luts {
            light_env.as_mut().connect_lut(i, u, d);
        }
        light_env
            .as_mut()
            .set_fresnel(citro3d::light::FresnelSelector::Both);
        light_env.as_mut().set_material(citro_mat);
    }

    pub fn draw(&mut self, prim: Primitive, verts: VboSlice<'f, '_>) {
        unsafe {
            self.gpu.draw(prim, verts.slice);
        }
    }
    pub fn draw_indexed(
        &mut self,
        prim: Primitive,
        buf: &VboSlice<'f, '_>,
        indices: &'f LinearBuffer<u16>,
    ) {
        unsafe {
            self.gpu
                .inst()
                .draw_elements(prim, buf.slice.info(), indices.deref());
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

fn bevy_to_citro3d_clr(c: Color) -> citro3d::material::Color {
    citro3d::material::Color::new(c.r(), c.g(), c.b())
}
