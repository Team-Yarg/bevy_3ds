use bevy::{render::{mesh::Mesh, render_asset::RenderAsset, render_resource::IndexFormat}, asset::Asset};
use ctru::linear::LinearAllocator;

use crate::gpu_buffer::LinearBuffer;

use self::gpu::{BufKind, GpuMesh};

use super::prep_asset::PrepareAsset;

pub mod gpu;
pub mod plugin;
pub mod systems;

#[derive(Asset)]
struct GpuMeshProxy(Mesh);

impl RenderAsset for GpuMeshProxy {
    type ExtractedAsset = Mesh;
    type PreparedAsset = GpuMesh;
    type Param = ();

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.0.clone()
    }

    fn prepare_asset(
        mesh: Self::ExtractedAsset,
        param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
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
                    index_buf: idx.to_vec_in(LinearAllocator),
                    nb: mesh.indices().unwrap().len() as u32,
                }
            })
            .unwrap_or(BufKind::Array);

        Ok(GpuMesh {
            vert_buf: vbo.as_slice().to_vec_in(LinearAllocator),
            nb_verts: mesh.count_vertices() as u32,
            indices: indecies,
            prim_kind: mesh.primitive_topology(),
        })
        unimplemented!()
    }
}
