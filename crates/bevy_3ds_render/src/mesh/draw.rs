use bevy::{
    asset::Handle,
    ecs::system::{lifetimeless::SRes, Query, Res},
    pbr::StandardMaterial,
    render::{mesh::Mesh, texture::Image, view::ExtractedView},
};
use citro3d::{macros::include_shader, texenv::Stage};
use lazy_static::lazy_static;
use log::debug;

use crate::{
    material::Uniforms,
    materials::RenderMaterials,
    mesh::{gpu::MeshVertex, plugin::ExtractedMesh},
    pass::{RenderCommand, VboBuffer},
    pipeline::VertexAttrs,
    shader::PicaShader,
    texture::BLANK_TEXTURE,
    vertattr::VertAttrBuilder,
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
        SRes<ExtractedMeshes>,
    );

    fn render<'w: 'f, 'f>(
        (meshes, images, assets, query): (
            Res<'w, RenderAssets<Mesh>>,
            Res<'w, RenderAssets<Image>>,
            Res<'w, RenderMaterials>,
            Res<ExtractedMeshes>,
        ),
        pass: &mut crate::pass::RenderPass<'w, 'f>,
        view: &ExtractedView,
        cam: CameraID,
    ) -> Result<(), crate::pass::RenderError> {
        let meshes = meshes.into_inner();
        let images = images.into_inner();

        pass.set_vertex_shader(&MESH_SHADER, 0)
            .expect("failed to set mesh shader");
        let uniforms = Uniforms::build(&MESH_SHADER);
        uniforms.bind_views(pass, view);

        let mut curr_mat: Option<&Handle<StandardMaterial>> = None;

        for ExtractedMesh {
            mesh: mesh_handle,
            transform,
            material: material_handle,
            render_on: render,
        } in &query.extracted
        {
            if !render.should_render_in(cam) {
                continue;
            }

            debug!("draw: {mesh_handle:?}");
            let mat_updated = curr_mat != Some(material_handle);
            if mat_updated {
                curr_mat.replace(material_handle);
            }
            let Some(mesh) = meshes.get(mesh_handle) else {
                debug!("mesh not loaded yet: {:?}", mesh_handle);
                continue;
            };

            let Some(material) = assets.get(material_handle) else {
                debug!("material not loaded yet: {:?}", material_handle);
                continue;
            };

            let tex = images
                .get(
                    material
                        .base_color_texture
                        .as_ref()
                        .unwrap_or(&BLANK_TEXTURE),
                )
                .unwrap_or_else(|| images.get(&BLANK_TEXTURE).unwrap());

            pass.bind_texture(0, tex);

            let norm = material
                .normal_map_texture
                .as_ref()
                .and_then(|n| images.get(n));

            if let Some(n) = norm {
                debug!("bind normal map for mesh");
                pass.bind_texture(1, n);
                pass.bind_normal_map(1);
            } else {
                pass.unbind_normal_map();
            }

            pass.configure_texenv(Stage::new(0).unwrap(), |s0| {
                s0.reset();
                s0.src(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::Source::Texture0,
                    Some(citro3d::texenv::Source::FragmentPrimaryColor),
                    None,
                )
                .func(
                    citro3d::texenv::Mode::RGB,
                    citro3d::texenv::CombineFunc::Modulate,
                );
            });
            pass.configure_texenv(Stage::new(1).unwrap(), |s1| {
                s1.reset();
                s1.src(
                    citro3d::texenv::Mode::RGB,
                    citro3d::texenv::Source::Previous,
                    Some(citro3d::texenv::Source::FragmentSecondaryColor),
                    None,
                )
                .func(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::CombineFunc::Add,
                );
            });

            if mat_updated {
                pass.set_lighting_material(material.to_owned().into());
            }
            //mat.set_uniforms(pass, &uniforms);
            pass.bind_vertex_uniform(uniforms.model_matrix, *transform);

            let mut buf = VboBuffer::new();
            let vbo = buf
                .add(&mesh.vert_buf, &MeshVertex::vert_attrs())
                .expect("failed to add vbo data");

            pass.set_attr_info(&VertexAttrs::from_citro3d(MeshVertex::vert_attrs()));
            match &mesh.indices {
                crate::mesh::gpu::BufKind::Array => {
                    pass.draw(mesh.prim_kind, vbo);
                }
                crate::mesh::gpu::BufKind::Elements { index_buf } => {
                    pass.draw_indexed(mesh.prim_kind, &vbo, index_buf);
                }
            }
        }

        Ok(())
    }
}
