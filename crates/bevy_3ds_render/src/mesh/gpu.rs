use crate::gpu_buffer::LinearBuffer;
use bevy::render::render_resource::PrimitiveTopology;

pub enum BufKind {
    Array,
    Elements {
        index_buf: LinearBuffer<u8>,
        nb: u32,
    },
}

pub struct GpuMesh {
    pub vert_buf: LinearBuffer<u8>,
    pub nb_verts: u32,
    pub indices: BufKind,
    pub prim_kind: PrimitiveTopology,
}
