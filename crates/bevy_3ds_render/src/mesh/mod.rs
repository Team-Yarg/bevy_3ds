use bevy::{
    app::Plugin,
    math::{Vec2, Vec3},
    render::{
        mesh::Mesh, render_asset::PrepareAssetError, render_resource::IndexFormat, texture::Image,
        RenderApp,
    },
};
use bevy_3ds_core::util::without_render_app;

use crate::{bevy_topology_to_citro, gpu_buffer::LinearBuffer, mesh::gpu::MeshVertex};

use self::gpu::{BufKind, GpuMesh};

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

use citro3d::texture::{Tex, TexParams};
use image::EncodableLayout;
use log::{trace, warn};

mod draw;
pub mod gpu;
mod plugin;

pub use plugin::MeshPlugin;

impl PrepareAsset for Mesh {
    type PreparedAsset = GpuMesh;
    type Param = ();

    fn prepare_asset_3ds(
        mesh: Self::ExtractedAsset,
        _: &mut bevy::ecs::system::SystemParamItem<<Self as PrepareAsset>::Param>,
    ) -> Result<
        <Self as PrepareAsset>::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        println!("prep asset 3ds");
        let vbo = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("failed to get vertex positions")
            .as_float3()
            .expect("failed to convert positions")
            .iter()
            .zip(
                mesh.attribute(Mesh::ATTRIBUTE_UV_0)
                    .expect("failed to get vertex UVs")
                    .as_float3()
                    .expect("failed to convert UVs")
                    .iter(),
            )
            .map(|(pos, uv)| MeshVertex {
                pos: Vec3::new(pos[0], pos[1], pos[2]),
                uv: Vec2::new(uv[0], uv[1]),
            })
            .collect::<Vec<_>>();

        let indecies = mesh
            .get_index_buffer_bytes()
            .map(|idx| {
                assert_eq!(
                    IndexFormat::from(mesh.indices().unwrap()),
                    IndexFormat::Uint32,
                    "can't use non-u32 index format"
                );
                BufKind::Elements {
                    index_buf: LinearBuffer::new(idx),
                    nb: mesh.indices().unwrap().len() as u32,
                }
            })
            .unwrap_or(BufKind::Array);

        Ok(GpuMesh {
            vert_buf: LinearBuffer::new(vbo.as_slice()),
            nb_verts: mesh.count_vertices() as u32,
            indices: indecies,
            prim_kind: bevy_topology_to_citro(mesh.primitive_topology())
                .expect("unsupported primitive type"),
        })
    }
}

#[derive(Default)]
pub struct PbrPlugin {
    /// we proxy stuff to this but intercept calls to functions which try and reference stuff we don't support
    /// e.g. RenderDevice
    inner: bevy::pbr::PbrPlugin,
}

impl Plugin for PbrPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        self.inner.build(app);
        //app.add_plugins(PrepareAssetsPlugin::<Image>::default());
    }

    fn ready(&self, app: &bevy::prelude::App) -> bool {
        self.inner.ready(app)
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        without_render_app(app, |app| self.inner.finish(app))
    }

    fn cleanup(&self, app: &mut bevy::prelude::App) {
        self.inner.cleanup(app);
    }

    fn name(&self) -> &str {
        // prevent being loaded along with the normal one
        std::any::type_name::<PbrPlugin>()
    }

    fn is_unique(&self) -> bool {
        true
    }
}
