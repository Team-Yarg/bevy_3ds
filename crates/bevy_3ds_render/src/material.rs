use std::f32::consts::PI;

use bevy::{
    math::{Vec3, Vec4, Vec4Swizzles},
    render::{color::Color, view::ExtractedView},
};
use bevy_3ds_core::util::wgpu_projection_to_opengl;

use crate::{pass::RenderPass, shader::PicaShader};

use citro3d::{
    light::{LightLut, LightLutId, LutInput},
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
    pub(crate) luts: Vec<(LightLutId, LutInput, LightLut)>,
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
        let spec_base = 0.16 * value.reflectance * value.reflectance;
        let f_0 = spec_base * (1.0 - value.metallic) + base.xyz() * value.metallic;

        let r = value.perceptual_roughness.min(1.0).max(0.089);
        /// ok before you scream allow me to explain
        ///
        /// We are using the D0 channel for the D part of the PBR equation, with fresnel
        /// actually just for the translucency. The GPU does the geometric part for us
        /// and I \*think\*, _pray_ that it gives us G/4(l * n)(v * n)
        ///
        /// helpful references:
        /// - https://marmoset.co/posts/basic-theory-of-physically-based-rendering/
        /// - https://google.github.io/filament/Filament.html
        ///
        luts.push((
            LightLutId::D0,
            LutInput::NormalHalf,
            LightLut::from_fn(
                |x| ((r * r) / (PI * ((x * x) * ((r * r) - 1.0) + 1.0).powf(2.0))).min(1.0),
                false,
            ),
        ));
        luts.push((
            LightLutId::Fresnel,
            LutInput::CosPhi,
            LightLut::from_fn(|_| (1.0 - value.diffuse_transmission), false),
        ));

        let diffuse = Color::rgb(diffuse.x, diffuse.y, diffuse.z);
        let spec = Color::rgb(f_0.x, f_0.y, f_0.z);
        Self {
            ambient: Some(value.base_color * 0.1),
            diffuse: Some(diffuse),
            specular0: Some(spec),
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
