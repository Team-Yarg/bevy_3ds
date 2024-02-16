use bevy::{
    app::Plugin,
    math::{Vec2, Vec3},
    render::mesh::{Mesh, VertexAttributeValues},
};
use bevy_3ds_core::util::without_render_app;

use crate::{bevy_topology_to_citro, gpu_buffer::LinearBuffer, mesh::gpu::MeshVertex};

use self::gpu::{BufKind, GpuMesh};

use super::prep_asset::PrepareAsset;

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

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("failed to get vertex positions")
            .as_float3()
            .expect("failed to convert positions");
        let uvs = mesh.attribute(Mesh::ATTRIBUTE_UV_0).map(|uv| match uv {
            VertexAttributeValues::Float32x2(f) => f,
            _ => unreachable!("should've already been caught by bevy"),
        });
        let normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .map(|n| {
                n.as_float3()
                    .expect("normals not float3??")
                    .iter()
                    .map(|s| Vec3::from_array(*s))
                    .collect::<Vec<_>>()
            })
            .expect("cannot render mesh without normals");

        let tangents = mesh
            .attribute(Mesh::ATTRIBUTE_TANGENT)
            .map(|tan| match tan {
                VertexAttributeValues::Float32x4(t) => t
                    .iter()
                    .map(|s| Vec3::from_slice(&s[..3]))
                    .collect::<Vec<_>>(),
                _ => todo!(),
            });

        let mut vbo = vec![];
        for index in 0..positions.len() {
            let pos = positions[index];
            let uv = uvs.expect("require UVs for mesh")[index];
            let tan = if let Some(t) = &tangents {
                t[index]
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            };
            vbo.push(MeshVertex {
                pos: Vec3::new(pos[0], pos[1], pos[2]),
                uv: Vec2::new(uv[0], uv[1]),
                normal: normals[index],
                tangent: tan,
            });
        }

let indices = mesh
    .indices()
    .map(|i| match i {
        bevy::render::mesh::Indices::U16(u) => BufKind::Elements {
            index_buf: LinearBuffer::new(&u),
        },
        bevy::render::mesh::Indices::U32(u) => {
            // Convert u32 to u16 with a check for overflow.
            let u16_indices: Vec<u16> = u.iter().map(|&index| {
                if index <= u16::MAX as u32 {
                    index as u16
                } else {
                    panic!("Index value exceeds u16 maximum range, cannot convert to u16.");
                }
            }).collect();
            BufKind::Elements {
                index_buf: LinearBuffer::new(&u16_indices),
            }
        },
    })
    .unwrap_or(BufKind::Array);

        Ok(GpuMesh {
            vert_buf: LinearBuffer::new(vbo.as_slice()),
            nb_verts: mesh.count_vertices() as u32,
            indices: indices,
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
