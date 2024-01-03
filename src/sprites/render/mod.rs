use std::ops::Add;

use bevy::{
    asset::{AssetId, AssetServer},
    ecs::{
        component::Component,
        system::{
            lifetimeless::{Read, SRes},
            Commands, Query, Res, ResMut, Resource,
        },
    },
    math::{Mat4, Vec2, Vec3, Vec4},
    render::{render_resource::VertexAttribute, texture::Image},
    sprite::{ExtractedSprite, ExtractedSprites},
};
use citro3d::{attrib::Register, buffer, macros::include_shader, math::Matrix4};
use lazy_static::lazy_static;

use crate::{
    gpu_buffer::LinearBuffer,
    render::{
        pass::{RenderCommand, RenderPass},
        pipeline::{RenderPipelineDescriptor, VertexAttrs, VertexState},
        shader::PicaShader,
        GpuDevice, RenderAssets,
    },
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: Vec3,
    uv: Vec2,
    colour: Vec4,
}

impl Vertex {
    fn attr_info() -> citro3d::attrib::Info {
        let mut info = citro3d::attrib::Info::new();
        info.add_loader(Register::new(0).unwrap(), citro3d::attrib::Format::Float, 2);
        info.add_loader(Register::new(1).unwrap(), citro3d::attrib::Format::Float, 2);
        info.add_loader(Register::new(2).unwrap(), citro3d::attrib::Format::Float, 4);
        info
    }
}

struct SpriteInstance {
    transform: Mat4,
    verts: LinearBuffer<Vertex>,
    indexes: LinearBuffer<u32>,
}

/// A batch of sprites which all share the same image
#[derive(Component)]
struct SpriteBatch {
    image: AssetId<Image>,
    sprites: Vec<SpriteInstance>,
}

#[derive(Resource, Default)]
pub struct SpriteBatches {
    batches: Vec<SpriteBatch>,
}

pub(super) fn prepare_sprites(
    mut cmds: Commands,
    images: Res<RenderAssets<Image>>,
    sprites: Res<ExtractedSprites>,
    mut batches: ResMut<SpriteBatches>,
) {
    batches.batches.clear();
    for (id, sprite) in &sprites.sprites {
        let transform = sprite.transform.compute_matrix();
        let mut verts: LinearBuffer<Vertex> = LinearBuffer::new(&[]);
        //let colour = sprite.color.as_rgba_f32().into();
        let uv = sprite.anchor;
        /*verts.push(Vertex {
            pos: Vec3::new(0, 0),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec3::new(0, 1),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec3::new(1, 1),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec3::new(1, 0),
            uv,
            colour,
        });*/
        let mut indexes = LinearBuffer::new(&[0, 1, 2, 0, 2, 1]);
        let batch = SpriteBatch {
            image: sprite.image_handle_id,
            sprites: vec![SpriteInstance {
                verts,
                indexes,
                transform,
            }],
        };
        batches.batches.push(batch);
    }
}
const SHADER_BYTES: &[u8] = include_shader!("./sprite.pica");

lazy_static! {
    static ref SPRITE_SHADER: PicaShader =
        PicaShader::load_from_bytes(SHADER_BYTES).expect("failed to load sprite shader");
}

fn draw_triangle(p: &mut RenderPass) {
    let verts = LinearBuffer::new(&[
        Vertex {
            pos: Vec3::new(-0.5, 0.5, 0.0),
            colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
            uv: Vec2::new(0., 0.),
        },
        Vertex {
            pos: Vec3::new(-0.5, -0.5, 0.0),
            colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
            uv: Vec2::new(0., 0.),
        },
        Vertex {
            pos: Vec3::new(0.5, -0.5, 0.0),
            colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
            uv: Vec2::new(0., 0.),
        },
        Vertex {
            pos: Vec3::new(0.5, 0.5, 0.0),
            colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
            uv: Vec2::new(0., 0.),
        },
    ]);
    p.set_pipeline(RenderPipelineDescriptor {
        label: Some("triangle"),
        vertex: VertexState {
            shader: &SPRITE_SHADER,
            entry_point: 0,
            attrs: VertexAttrs::from_citro3d(Vertex::attr_info()),
        },
    })
    .expect("failed to set triangle pipeline");
    let vbo = p
        .add_vertex_buffer(&verts)
        .expect("failed to set vertex buffer");
    p.draw(&vbo, buffer::Primitive::TriangleFan);
}

pub struct DrawSprites;

impl RenderCommand for DrawSprites {
    type Param = SRes<SpriteBatches>;

    fn render<'w, 'f, 'g>(
        entity: Res<'w, SpriteBatches>,
        pass: &'f mut RenderPass<'g>,
    ) -> Result<(), crate::render::pass::RenderError> {
        for sprite in &entity.batches {
            draw_triangle(pass);
        }
        Ok(())
    }
}
