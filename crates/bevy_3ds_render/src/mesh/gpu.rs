use crate::{
    gpu_buffer::LinearBuffer,
    vertattr::{VertAttrBuilder, VertAttrs},
};
use bevy::math::{Vec2, Vec3};
use citro3d::buffer::Primitive;

pub enum BufKind {
    Array,
    Elements {
        index_buf: LinearBuffer<u8>,
        nb: u32,
    },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, VertAttrBuilder)]
pub struct MeshVertex {
    pub pos: Vec3,
    pub uv: Vec2,
}

pub struct GpuMesh {
    pub vert_buf: LinearBuffer<MeshVertex>,
    pub nb_verts: u32,
    pub indices: BufKind,
    pub prim_kind: Primitive,
}
