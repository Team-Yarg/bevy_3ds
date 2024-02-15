use bevy::{
    app::Plugin,
    math::{Vec2, Vec3},
    render::{
        mesh::{Mesh, VertexAttributeValues},
        render_resource::{VertexBufferLayout, VertexFormat},
    },
};
use bevy_3ds_core::util::without_render_app;

use crate::{
    bevy_topology_to_citro, gpu_buffer::LinearBuffer, mesh::gpu::MeshVertex, pipeline::VertexAttrs,
};

use self::gpu::{BufKind, GpuMesh};

use super::prep_asset::{PrepareAsset, PrepareAssetsPlugin};

use citro3d::{
    attrib::{self, Format, Register},
    texture::{Tex, TexParams},
};

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

        fn conv_vert_attrs(layout: &VertexBufferLayout) -> attrib::Info {
            let mut attrs = attrib::Info::new();
            for attr in &layout.attributes {
                let reg = Register::new(attr.shader_location.try_into().unwrap())
                    .expect("invalid shader location for attribute");
                let ubyte_fallback = |f: VertexFormat| (Format::UnsignedByte, f.size());
                let (fmt, sz) = match attr.format {
                    bevy::render::render_resource::VertexFormat::Uint8x2 => {
                        (Format::UnsignedByte, 2)
                    }
                    bevy::render::render_resource::VertexFormat::Uint8x4 => {
                        (Format::UnsignedByte, 4)
                    }
                    bevy::render::render_resource::VertexFormat::Sint8x2 => (Format::Byte, 2),
                    bevy::render::render_resource::VertexFormat::Sint8x4 => (Format::Byte, 4),
                    bevy::render::render_resource::VertexFormat::Sint16x2 => (Format::Short, 2),
                    bevy::render::render_resource::VertexFormat::Sint16x4 => (Format::Short, 4),
                    bevy::render::render_resource::VertexFormat::Float32 => (Format::Float, 1),
                    bevy::render::render_resource::VertexFormat::Float32x2 => (Format::Float, 2),
                    bevy::render::render_resource::VertexFormat::Float32x3 => (Format::Float, 3),
                    bevy::render::render_resource::VertexFormat::Float32x4 => (Format::Float, 4),
                    bevy::render::render_resource::VertexFormat::Uint32
                    | bevy::render::render_resource::VertexFormat::Uint32x2
                    | bevy::render::render_resource::VertexFormat::Uint32x3
                    | bevy::render::render_resource::VertexFormat::Uint32x4 => {
                        ubyte_fallback(attr.format)
                    }

                    f => {
                        unimplemented!("GPU does not support {f:?} attributes")
                    }
                };
                attrs
                    .add_loader(reg, fmt, sz as u8)
                    .expect("failed to register loader for attr");
            }
            attrs
        }

        assert!(
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some(),
            "GPU requires UVs to render meshes correctly"
        );
        let vbo = mesh.get_vertex_buffer_data();
        let vbo_layout = mesh.get_mesh_vertex_buffer_layout();
        let vert_layout = vbo_layout.layout();
        let vert_attributes = conv_vert_attrs(vert_layout);

        let indecies = mesh
            .indices()
            .map(|i| match i {
                bevy::render::mesh::Indices::U16(u) => BufKind::Elements {
                    index_buf: LinearBuffer::new(&u),
                },
                bevy::render::mesh::Indices::U32(i) => panic!("can't use 32bit indices"),
            })
            .unwrap_or(BufKind::Array);

        Ok(GpuMesh {
            vert_buf: LinearBuffer::new(vbo.as_slice()),
            vert_attributes: VertexAttrs::from_citro3d(vert_attributes),
            nb_verts: mesh.count_vertices() as u32,
            vert_stride: vert_layout.array_stride as u32,
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
