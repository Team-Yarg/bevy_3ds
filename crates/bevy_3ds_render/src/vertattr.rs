use bevy::math::{Vec2, Vec3, Vec4};
pub use bevy_3ds_macros::*;
use citro3d::attrib::Format::*;

pub trait VertAttrs {
    const FORMAT: citro3d::attrib::Format;
    const SIZE: u8;
}

pub trait VertAttrBuilder {
    fn vert_attrs() -> citro3d::attrib::Info;
}

macro_rules! impl_vertattrs {
    ($t:tt: $f:expr, $s:literal) => {
        impl VertAttrs for $t {
            const FORMAT: citro3d::attrib::Format = $f;
            const SIZE: u8 = $s;
        }
    };
}

impl_vertattrs!(Vec2: Float, 2);
impl_vertattrs!(Vec3: Float, 3);
impl_vertattrs!(Vec4: Float, 4);

impl_vertattrs!(u32: UnsignedByte, 4);

impl_vertattrs!(f32: Float, 1);

impl_vertattrs!((f32,): Float, 1);
impl_vertattrs!((f32, f32): Float, 2);
impl_vertattrs!((f32, f32, f32): Float, 3);
impl_vertattrs!((f32, f32, f32, f32): Float, 4);

impl_vertattrs!([f32; 1]: Float, 1);
impl_vertattrs!([f32; 2]: Float, 2);
impl_vertattrs!([f32; 3]: Float, 3);
impl_vertattrs!([f32; 4]: Float, 4);

impl_vertattrs!(u8: UnsignedByte, 1);

impl_vertattrs!((u8,): UnsignedByte, 1);
impl_vertattrs!((u8, u8): UnsignedByte, 2);
impl_vertattrs!((u8, u8, u8): UnsignedByte, 3);
impl_vertattrs!((u8, u8, u8, u8): UnsignedByte, 4);

impl_vertattrs!([u8; 1]: UnsignedByte, 1);
impl_vertattrs!([u8; 2]: UnsignedByte, 2);
impl_vertattrs!([u8; 3]: UnsignedByte, 3);
impl_vertattrs!([u8; 4]: UnsignedByte, 4);

impl_vertattrs!(i8: Byte, 1);

impl_vertattrs!((i8,): Byte, 1);
impl_vertattrs!((i8, i8): Byte, 2);
impl_vertattrs!((i8, i8, i8): Byte, 3);
impl_vertattrs!((i8, i8, i8, i8): Byte, 4);

impl_vertattrs!([i8; 1]: Byte, 1);
impl_vertattrs!([i8; 2]: Byte, 2);
impl_vertattrs!([i8; 3]: Byte, 3);
impl_vertattrs!([i8; 4]: Byte, 4);

impl_vertattrs!(i16: Short, 1);

impl_vertattrs!((i16,): Short, 1);
impl_vertattrs!((i16, i16): Short, 2);
impl_vertattrs!((i16, i16, i16): Short, 3);
impl_vertattrs!((i16, i16, i16, i16): Short, 4);

impl_vertattrs!([i16; 1]: Short, 1);
impl_vertattrs!([i16; 2]: Short, 2);
impl_vertattrs!([i16; 3]: Short, 3);
impl_vertattrs!([i16; 4]: Short, 4);
