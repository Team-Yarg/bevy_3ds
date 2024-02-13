use bevy::{
    math::{Vec4, Vec4Swizzles},
    render::{color::Color, view::ExtractedView},
};
use bevy_3ds_core::util::wgpu_projection_to_opengl;

use crate::{pass::RenderPass, shader::PicaShader};

use citro3d::{
    light::{LightLutId, LutData, LutInput},
    math::FVec4,
    uniform::{Index, Uniform},
};

#[derive(Default)]
pub struct Material {
    pub ambient: Option<Color>,
    pub diffuse: Option<Color>,
    pub specular0: Option<Color>,
    pub specular1: Option<Color>,
    pub emission: Option<Color>,
    pub(crate) luts: Vec<(LightLutId, LutInput, LutData)>,
}

impl Material {
    pub fn new(
        ambient: Option<Color>,
        diffuse: Option<Color>,
        specular0: Option<Color>,
        specular1: Option<Color>,
        emission: Option<Color>,
    ) -> Self {
        Self {
            ambient,
            diffuse,
            specular0,
            specular1,
            emission,
            luts: Default::default(),
        }
    }
}

impl From<bevy::pbr::StandardMaterial> for Material {
    fn from(value: bevy::pbr::StandardMaterial) -> Self {
        let mut luts = Vec::new();
        let base: Vec4 = value.base_color.into();
        let emissive: Vec4 = value.emissive.into();
        let diffuse = base.xyz()
            * (1.0 - value.metallic)
            * (1.0 - value.specular_transmission)
            * (1.0 - value.diffuse_transmission);
        let emissive = emissive.xyz() * base.w;
        let f_0 = 0.16 * value.reflectance * value.reflectance * (1.0 - value.metallic)
            + base.xyz() * value.metallic;
        let spec_base = 0.16 * value.reflectance * value.reflectance;
        if value.metallic == 0.0 {
            let spec_exponent = 1.0 / spec_base;
            luts.push((
                LightLutId::D0,
                LutInput::LightNormal,
                LutData::from_fn(|x| x.powf(spec_exponent), false),
            ));
        }
        luts.push((
            LightLutId::Fresnel,
            LutInput::NormalView,
            LutData::from_fn(|x| x, false),
        ));

        luts.push((
            LightLutId::ReflectRed,
            LutInput::NormalView,
            LutData::from_fn(|x| (x * f_0).x, false),
        ));

        luts.push((
            LightLutId::ReflectGreen,
            LutInput::NormalView,
            LutData::from_fn(|x| (x * f_0).y, false),
        ));

        luts.push((
            LightLutId::ReflectBlue,
            LutInput::NormalView,
            LutData::from_fn(|x| (x * f_0).z, false),
        ));
        Self {
            ambient: None,
            diffuse: Some(Color::rgb(diffuse.x, diffuse.y, diffuse.z)),
            specular0: Some(Color::WHITE),
            specular1: None,
            emission: Some(Color::rgb(emissive.x, emissive.y, emissive.z)),
            luts,
        }
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
