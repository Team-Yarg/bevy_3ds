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
use citro3d::{
    buffer,
    macros::include_shader,
    math::{ClipPlanes, Matrix4, Perspective, Projection},
    texenv::Stage,
};
use lazy_static::lazy_static;
use log::debug;

use crate::{
    material::{Material, Uniforms},
    materials::RenderMaterials,
    mesh::{gpu::MeshVertex, plugin::ExtractedMesh},
    pass::{RenderCommand, VboBuffer},
    pipeline::VertexAttrs,
    shader::PicaShader,
    vertattr::{VertAttrBuilder, VertAttrs},
    RenderAssets,
};

use super::plugin::ExtractedMeshes;

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
        SRes<RenderMaterials>,
        Query<'static, 'static, &'static ExtractedView>,
        SRes<ExtractedMeshes>,
    );

    fn render<'w: 'f, 'f>(
        (meshes, images, assets, views, query): (
            Res<'w, RenderAssets<Mesh>>,
            Res<'w, RenderAssets<Image>>,
            Res<'w, RenderMaterials>,
            Query<&ExtractedView>,
            Res<ExtractedMeshes>,
        ),
        pass: &mut crate::pass::RenderPass<'w, 'f>,
        view_id: bevy::prelude::Entity,
    ) -> Result<(), crate::pass::RenderError> {
        let meshes = meshes.into_inner();
        let images = images.into_inner();
        let view = views.get(view_id).expect("failed to find view for draw");

        pass.set_vertex_shader(&MESH_SHADER, 0)
            .expect("failed to set mesh shader");

        let uniforms = Uniforms::build(&MESH_SHADER);
        let view_proj = wgpu_projection_to_opengl(view.projection);
        let proj = Projection::perspective(
            40.,
            citro3d::math::AspectRatio::TopScreen,
            ClipPlanes {
                near: 0.01,
                far: 1000.,
            },
        );
        //pass.bind_vertex_uniform(uniforms.projection_matrix, &proj.into());
        //pass.bind_vertex_uniform(uniforms.camera_matrix, &Matrix4::identity());

        pass.bind_vertex_uniform_bevy(uniforms.projection_matrix, &view_proj);
        pass.bind_vertex_uniform_bevy(uniforms.camera_matrix, &view.transform.compute_matrix());

        debug!("draw mesh");

        for ExtractedMesh {
            mesh: mesh_handle,
            transform,
            material: material_handle,
        } in &query.extracted
        {
            debug!("draw: {mesh_handle:?}");
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
            let mat = Material::new(Some(material.base_color), None);
            mat.set_uniforms(pass, &uniforms);

            pass.bind_vertex_uniform_bevy(uniforms.model_matrix, transform);
            let verts = mesh
                .vert_buf
                .iter()
                .map(|v| {
                    view.projection.transform_point3(
                        view.transform
                            .compute_matrix()
                            .transform_point3(transform.transform_point3(v.pos)),
                    )
                })
                .collect::<Vec<_>>();
            debug!("verts: {verts:#?}");

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
