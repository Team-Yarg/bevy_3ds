use bevy::{
    math::Vec4,
    render::{color::Color, view::ExtractedView},
};
use bevy_3ds_core::util::wgpu_projection_to_opengl;

use crate::{pass::RenderPass, shader::PicaShader};

use citro3d::{
    math::FVec4,
    uniform::{Index, Uniform},
};

#[derive(Debug, Default)]
pub struct Material {
    colour: Option<Color>,
    ambient: Option<Color>,
}

impl Material {
    pub fn new(colour: Option<Color>, ambient: Option<Color>) -> Self {
        Self { colour, ambient }
    }

    pub fn set_uniforms(&self, pass: &mut RenderPass, uniforms: &Uniforms) {
        let amb = if let Some(clr) = &self.ambient {
            clr.as_rgba_f32().into()
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };

        let emi = if let Some(clr) = &self.colour {
            clr.as_rgba_f32().into()
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };
        pass.bind_vertex_uniform(
            uniforms.material_ambient,
            Uniform::Float(FVec4::new(amb.x, amb.y, amb.z, amb.w)),
        );

        pass.bind_vertex_uniform(
            uniforms.material_emission,
            Uniform::Float(FVec4::new(emi.x, emi.y, emi.z, emi.w)),
        );
    }
}

pub struct Uniforms {
    pub model_matrix: Index,
    pub camera_matrix: Index,
    pub projection_matrix: Index,
    pub light_colour: Index,
    pub material_emission: Index,
    pub material_ambient: Index,
    pub material_diffuse: Index,
    pub material_specular: Index,
}

impl Uniforms {
    pub fn build(vert_prog: &PicaShader) -> Self {
        let model_matrix = vert_prog.get_uniform("modelMtx").unwrap();
        let camera_matrix = vert_prog.get_uniform("camMtx").unwrap();
        let projection_matrix = vert_prog.get_uniform("projMtx").unwrap();

        let light_colour = vert_prog.get_uniform("lightClr").unwrap();

        let material_emission = vert_prog.get_uniform("mat_emi").unwrap();
        let material_ambient = vert_prog.get_uniform("mat_amb").unwrap();
        let material_diffuse = vert_prog.get_uniform("mat_dif").unwrap();
        let material_specular = vert_prog.get_uniform("mat_spe").unwrap();

        Uniforms {
            model_matrix,
            camera_matrix,
            projection_matrix,
            light_colour,
            material_emission,
            material_ambient,
            material_diffuse,
            material_specular,
        }
    }
    pub fn bind_views(&self, pass: &mut RenderPass, view: &ExtractedView) {
        let view_proj = wgpu_projection_to_opengl(view.projection);
        pass.bind_vertex_uniform(self.projection_matrix, view_proj);
        pass.bind_vertex_uniform(
            self.camera_matrix,
            view.transform.compute_matrix().inverse(),
        );
    }
}
