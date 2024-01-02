use bevy::{
    asset::Asset,
    render::{mesh::Mesh, render_asset::RenderAsset, render_resource::IndexFormat},
};
use ctru::linear::LinearAllocator;

use crate::gpu_buffer::LinearBuffer;

use self::gpu::{BufKind, GpuMesh};

use super::prep_asset::PrepareAsset;

pub mod gpu;
mod plugin;
pub mod systems;

pub use plugin::MeshPlugin;

impl PrepareAsset for Mesh {
    type PreparedAsset = GpuMesh;
    type Param = ();

    fn prepare_asset_3ds(
        mesh: Self::ExtractedAsset,
        param: &mut bevy::ecs::system::SystemParamItem<<Self as PrepareAsset>::Param>,
    ) -> Result<
        <Self as PrepareAsset>::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        println!("prep asset 3ds");
        let vbo = mesh.get_vertex_buffer_data();
        let indecies = mesh
            .get_index_buffer_bytes()
            .map(|idx| {
                assert_eq!(
                    IndexFormat::from(mesh.indices().unwrap()),
                    IndexFormat::Uint32,
                    "can't use non-u32 index format"
                );
                BufKind::Elements {
                    index_buf: LinearBuffer::new(&idx),
                    nb: mesh.indices().unwrap().len() as u32,
                }
            })
            .unwrap_or(BufKind::Array);

        Ok(GpuMesh {
            vert_buf: LinearBuffer::new(vbo.as_slice()),
            nb_verts: mesh.count_vertices() as u32,
            indices: indecies,
            prim_kind: mesh.primitive_topology(),
        })
    }
}
