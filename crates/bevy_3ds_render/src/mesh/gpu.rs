use bevy::render::render_resource::PrimitiveTopology;
use citro3d::math::{FVec3, FVec4};

use crate::gpu_buffer::LinearBuffer;

pub struct Vertex {
    pos: FVec4,
    tex: FVec3,
}

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
