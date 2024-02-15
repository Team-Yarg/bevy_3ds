use crate::{
    gpu_buffer::LinearBuffer,
    pipeline::VertexAttrs,
    vertattr::{VertAttrBuilder, VertAttrs},
};
use bevy::math::{Vec2, Vec3};
use citro3d::buffer::Primitive;

pub enum BufKind {
    Array,
    Elements { index_buf: LinearBuffer<u16> },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, VertAttrBuilder)]
pub struct MeshVertex {
    pub pos: Vec3,
    pub uv: Vec2,
}

pub struct GpuMesh {
    pub vert_buf: LinearBuffer<u8>,
    pub nb_verts: u32,
    pub vert_stride: u32,
    pub vert_attributes: VertexAttrs,
    pub indices: BufKind,
    pub prim_kind: Primitive,
}
