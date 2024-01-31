use std::f32::consts::TAU;

use bevy::{
    asset::{Assets, Handle},
    ecs::system::{lifetimeless::SRes, Query, Res},
    math::{Mat4, Vec3},
    pbr::StandardMaterial,
    render::{mesh::Mesh, texture::Image, view::ExtractedView},
    transform::components::GlobalTransform,
};
use bevy_3ds_core::util::wgpu_projection_to_opengl;
use citro3d::{buffer, macros::include_shader, math::Matrix4, texenv::Stage};
use lazy_static::lazy_static;
use log::debug;

use crate::{
    material::Uniforms,
    mesh::gpu::MeshVertex,
    pass::{RenderCommand, VboBuffer},
    pipeline::VertexAttrs,
    shader::PicaShader,
    vertattr::{VertAttrBuilder, VertAttrs},
    RenderAssets,
};

const SHADER_BYTES: &[u8] = include_shader!("./mesh.pica");

lazy_static! {
    static ref MESH_SHADER: PicaShader =
        PicaShader::load_from_bytes(SHADER_BYTES).expect("failed to load mesh shader");
}

pub struct MeshDraw;

impl RenderCommand for MeshDraw {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SRes<RenderAssets<Image>>,
        SRes<Assets<StandardMaterial>>,
        Query<'static, 'static, &'static ExtractedView>,
        Query<
            'static,
            'static,
            (
                &'static Handle<Mesh>,
                &'static Handle<StandardMaterial>,
                &'static GlobalTransform,
            ),
        >,
    );

    fn render<'w: 'f, 'f>(
        (meshes, images, assets, views, query): (
            Res<'w, RenderAssets<Mesh>>,
            Res<'w, RenderAssets<Image>>,
            Res<'w, Assets<StandardMaterial>>,
            Query<&ExtractedView>,
            Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform)>,
        ),
        pass: &mut crate::pass::RenderPass<'w, 'f>,
        view_id: bevy::prelude::Entity,
    ) -> Result<(), crate::pass::RenderError> {
        let meshes = meshes.into_inner();
        let images = images.into_inner();
        let view = views.get(view_id).expect("failed to find view for draw");

        let mut camera_matrix = Matrix4::identity();

        camera_matrix.translate(0.0, 0.0, -1.0); // wgpu to opengl translation
        pass.set_vertex_shader(&MESH_SHADER, 0)
            .expect("failed to set mesh shader");

        let mut view_proj = wgpu_projection_to_opengl(view.projection);
        view_proj *= Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), TAU / 4.0);

        let uniforms = Uniforms::build(&MESH_SHADER);
        pass.bind_vertex_uniform(uniforms.camera_matrix, &camera_matrix);

        pass.set_attr_info(&VertexAttrs::from_citro3d(MeshVertex::vert_attrs()));

        let view_uniform = MESH_SHADER.get_uniform("projMtx").unwrap();
        pass.bind_vertex_uniform_bevy(view_uniform, &view_proj);

        debug!("draw mesh");

        for (mesh_handle, material_handle, transform) in &query {
            let Some(mesh) = meshes.get(mesh_handle) else {
                debug!("mesh not loaded yet: {:?}", mesh_handle);
                continue;
            };

            let Some(material) = assets.get(material_handle) else {
                debug!("material not loaded yet: {:?}", material_handle);
                continue;
            };

            let tex = material
                .base_color_texture
                .as_ref()
                .map_or(None, |t| images.get(t));

            let uses_tex = if let Some(t) = tex {
                debug!("bind texture for mesh");
                pass.bind_texture(0, t);
                true
            } else {
                false
            };

            pass.configure_texenv(Stage::new(0).unwrap(), |s0| {
                if uses_tex {
                    s0.reset();
                    s0.src(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::Source::Texture0,
                        None,
                        None,
                    )
                    .func(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::CombineFunc::Replace,
                    );
                } else {
                    s0.reset();
                    s0.src(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::Source::PrimaryColor,
                        None,
                        None,
                    )
                    .func(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::CombineFunc::Replace,
                    );
                }
            });

            pass.bind_vertex_uniform_bevy(uniforms.model_matrix, &transform.compute_matrix());
            debug!("transform: {:#?}", transform);

            let mut buf = VboBuffer::new();
            let vbo = buf
                .add(&mesh.vert_buf, &MeshVertex::vert_attrs())
                .expect("failed to add vbo data");

            pass.set_attr_info(&VertexAttrs::from_citro3d(MeshVertex::vert_attrs()));

            pass.draw(mesh.prim_kind, vbo);
        }

        Ok(())
    }
}
