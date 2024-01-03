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
    math::Mat4,
    render::{render_resource::VertexAttribute, texture::Image, view},
    sprite::{ExtractedSprite, ExtractedSprites},
};
use citro3d::{
    attrib::Register,
    buffer,
    macros::include_shader,
    math::{AspectRatio, ClipPlanes, Matrix4, Projection},
    texenv::Stage,
};
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
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: Vec3,
    colour: Vec4,
    uv: Vec2,
}

impl Vertex {
    fn attr_info() -> citro3d::attrib::Info {
        let mut info = citro3d::attrib::Info::new();
        info.add_loader(Register::new(0).unwrap(), citro3d::attrib::Format::Float, 3)
            .unwrap();
        info.add_loader(Register::new(1).unwrap(), citro3d::attrib::Format::Float, 4)
            .unwrap();
        info.add_loader(Register::new(2).unwrap(), citro3d::attrib::Format::Float, 2)
            .unwrap();
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
            /*Vertex {
                pos: Vec3::new(0.5, 0.5, 0.0),
                colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
                uv: Vec2::new(0., 0.),
            },*/
        ]);
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

fn draw_triangle(p: &mut RenderPass, verts: &LinearBuffer<Vertex>) {
    log::debug!("draw triangle");
    let model_uniform = SPRITE_SHADER.get_uniform("modelMtx").unwrap();
    let mut buf = citro3d::buffer::Info::new();
    let vbo = buf
        .add(&verts, &Vertex::attr_info())
        .expect("failed to add vbo data");
    let mut transform = Matrix4::identity();
    transform.scale(3., 3., 3.);
    p.bind_vertex_uniform(model_uniform, &transform);
    p.draw(buffer::Primitive::Triangles, vbo);

    log::debug!("draw triangle fin");
}

fn calculate_projections() -> Matrix4 {
    // TODO: it would be cool to allow playing around with these parameters on
    // the fly with D-pad, etc.
    let slider_val = ctru::os::current_3d_slider_state();
    let interocular_distance = slider_val / 2.0;

    let vertical_fov = 40.0_f32.to_radians();
    let screen_depth = 2.0;

    let clip_planes = ClipPlanes {
        near: 0.01,
        far: 100.0,
    };

    let proj = Projection::perspective(vertical_fov, AspectRatio::TopScreen, clip_planes).into();
    proj
}

pub struct DrawSprites;

impl RenderCommand for DrawSprites {
    type Param = SRes<SpriteBatches>;

    fn render<'w, 'f, 'g>(
        entity: Res<'w, SpriteBatches>,
        pass: &'f mut RenderPass<'g>,
    ) -> Result<(), crate::render::pass::RenderError> {
        pass.set_vertex_shader(&SPRITE_SHADER, 0)
            .expect("failed to set sprite shader");
        pass.set_attr_info(&VertexAttrs::from_citro3d(Vertex::attr_info()));
        let view_uniform = SPRITE_SHADER.get_uniform("projMtx").unwrap();
        pass.bind_vertex_uniform(view_uniform, &calculate_projections());
        pass.configure_texenv(Stage::new(0).unwrap(), |s0| {
            s0.reset();
            s0.src(
                citro3d::texenv::Mode::BOTH,
                citro3d::texenv::Source::PrimaryColor,
                None,
                None,
            )
            .func(
                citro3d::texenv::Mode::BOTH,
                citro3d::texenv::CombineFunc::Replace,
            );
        });

        for sprite in &entity.batches {
            for s in &sprite.sprites {
                draw_triangle(pass, &s.verts);
            }
        }
        Ok(())
    }
}
