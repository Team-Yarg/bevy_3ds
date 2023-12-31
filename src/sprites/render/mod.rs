use std::ops::Add;

use bevy::{
    asset::AssetId,
    ecs::{
        component::Component,
        system::{
            lifetimeless::{Read, SRes},
            Commands, Query, Res, ResMut, Resource,
        },
    },
    math::{Vec2, Vec3, Vec4},
    render::texture::Image,
    sprite::{ExtractedSprite, ExtractedSprites},
};
use citro3d::{attrib::Register, buffer, math::Matrix4};

use crate::{
    gpu_buffer::LinearBuffer,
    render::{pass::RenderCommand, GpuDevice, RenderAssets},
};

/// Holds all the rendering information for every sprite in this pass
#[derive(Resource)]
struct SpriteRenderInfo {}

#[repr(C)]
struct Vertex {
    pos: Vec2,
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
    transform: Matrix4,
    verts: LinearBuffer<Vertex>,
    indexes: LinearBuffer<u32>,
}

/// A batch of sprites which all share the same image
#[derive(Component)]
struct SpriteBatch {
    image: AssetId<Image>,
    sprites: Vec<SpriteInstance>,
}

fn prepare_sprites(
    mut cmds: Commands,
    images: Res<RenderAssets<Image>>,
    sprites: Res<ExtractedSprites>,
    render_info: ResMut<SpriteRenderInfo>,
) {
    for (id, sprite) in &sprites.sprites {
        let sprite: ExtractedSprite = sprite;
        let transform = sprite.transform.compute_matrix();
        let mut verts = LinearBuffer::new();
        let colour = sprite.color.as_rgba_f32().into();
        let uv = sprite.anchor;
        verts.push(Vertex {
            pos: Vec2::new(0, 0),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec2::new(0, 1),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec2::new(1, 1),
            uv,
            colour,
        });
        verts.push(Vertex {
            pos: Vec2::new(1, 0),
            uv,
            colour,
        });
        let mut indexes = LinearBuffer::new();
        indexes.push(0);
        indexes.push(1);
        indexes.push(2);

        indexes.push(0);
        indexes.push(2);
        indexes.push(1);
        let batch = SpriteBatch {
            image: sprite.image_handle_id,
            sprites: vec![SpriteInstance {
                verts,
                indexes,
                transform,
            }],
        };
        cmds.spawn(batch);
    }
}

pub struct DrawSprites;

impl RenderCommand for DrawSprites {
    type Param = ();
    type ItemData = Read<SpriteBatch>;

    fn render<'w>(
        entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemData>,
        param: &bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut crate::render::pass::RenderPass,
    ) -> Result<(), crate::render::pass::RenderError> {
        todo!()
    }
}
