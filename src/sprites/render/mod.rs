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
    render::{color::Color, render_resource::VertexAttribute, texture::Image, view},
    sprite::{ExtractedSprite, ExtractedSprites},
};
use citro3d::{
    attrib::{self, Register},
    buffer::{self, Primitive},
    macros::include_shader,
    math::{AspectRatio, ClipPlanes, Matrix4, Projection},
    texenv::Stage,
    uniform::Index,
};
use lazy_static::lazy_static;

use crate::{
    gpu_buffer::LinearBuffer,
    render::{
        pass::{RenderCommand, RenderPass},
        pipeline::{RenderPipelineDescriptor, VertexAttrs, VertexState},
        shader::PicaShader,
        GpuDevice, GpuImage, RenderAssets,
    },
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: Vec3,
    //colour: Vec4,
    uv: Vec2,
}

impl Vertex {
    fn attr_info() -> citro3d::attrib::Info {
        let mut info = citro3d::attrib::Info::new();
        info.add_loader(Register::new(0).unwrap(), citro3d::attrib::Format::Float, 3)
            .unwrap();
        info.add_loader(Register::new(1).unwrap(), citro3d::attrib::Format::Float, 2)
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
                uv: Vec2::new(0., 0.),
            },
            Vertex {
                pos: Vec3::new(-0.5, -0.5, 0.0),
                uv: Vec2::new(0., 0.),
            },
            Vertex {
                pos: Vec3::new(0.5, -0.5, 0.0),
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
const SHADER_BYTES: &[u8] = include_shader!("./shader.pica");

lazy_static! {
    static ref SPRITE_SHADER: PicaShader =
        PicaShader::load_from_bytes(SHADER_BYTES).expect("failed to load sprite shader");
}

#[derive(Debug, Default)]
pub struct Material {
    texture: Option<AssetId<Image>>,
    colour: Option<Color>,
    ambient: Option<Color>,
    vertex_colours: bool,
}

impl Material {
    pub fn new(
        texture: Option<AssetId<Image>>,
        colour: Option<Color>,
        ambient: Option<Color>,
        vertex_colours: bool,
    ) -> Self {
        Self {
            texture,
            colour,
            ambient,
            vertex_colours,
        }
    }

    pub fn use_vertex_colours(&self) -> bool {
        self.vertex_colours
    }

    pub fn get_texture<'a>(&self, images: &'a RenderAssets<Image>) -> Option<&'a GpuImage> {
        self.texture.and_then(|id| images.get(id))
    }

    pub fn set_uniforms(&self, _gpu: &mut RenderPass, uniforms: &Uniforms) {
        let amb = if let Some(clr) = &self.ambient {
            clr.as_rgba_f32().into()
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };

        let emi = if let Some(clr) = &self.colour {
            clr.as_rgba_f32().into()
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };

        unsafe {
            citro3d_sys::C3D_FVUnifSet(
                citro3d::shader::Type::Vertex.into(),
                uniforms.material_ambient.into(),
                amb.x,
                amb.y,
                amb.z,
                amb.w,
            );
            citro3d_sys::C3D_FVUnifSet(
                citro3d::shader::Type::Vertex.into(),
                uniforms.material_emission.into(),
                emi.x,
                emi.y,
                emi.z,
                emi.w,
            );
        }
    }
}

pub struct Uniforms {
    pub model_matrix: Index,
    pub camera_matrix: Index,
    pub projection_matrix: Index,
    pub light_colour: Index,
    pub material_emission: Index,
    pub material_ambient: Index,
    pub material_diffuse: Index,
    pub material_specular: Index,
}

#[derive(Debug)]
pub struct Model<T> {
    pub pos: Vec3,
    pub rot: Vec3,
    shapes: Vec<Shape<T>>,
}

#[derive(Debug)]
pub struct Shape<T> {
    mat: Material,
    prim_type: Primitive,
    verts: LinearBuffer<T>,
    attr_info: attrib::Info,
}

fn draw_triangle(p: &mut RenderPass, verts: &LinearBuffer<Vertex>, uniforms: &Uniforms) {
    log::debug!("draw triangle");

    p.configure_texenv(Stage::new(0).unwrap(), |s0| {
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

    let mut buf = citro3d::buffer::Info::new();
    let vbo = buf
        .add(&verts, &Vertex::attr_info())
        .expect("failed to add vbo data");

    let mut transform = Matrix4::identity();
    transform.scale(3., 3., 3.);

    p.bind_vertex_uniform(uniforms.model_matrix, &transform);
    p.set_attr_info(&VertexAttrs::from_citro3d(Vertex::attr_info()));
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

fn build_uniforms() -> Uniforms {
    let vert_prog = &SPRITE_SHADER;
    let model_uniform = vert_prog.get_uniform("modelMtx").unwrap();
    let cam_uniform = vert_prog.get_uniform("camMtx").unwrap();
    let proj_uniform = vert_prog.get_uniform("projMtx").unwrap();

    let light_uniform = vert_prog.get_uniform("lightClr").unwrap();

    let emi_uniform = vert_prog.get_uniform("mat_emi").unwrap();
    let amb_uniform = vert_prog.get_uniform("mat_amb").unwrap();
    let dif_uniform = vert_prog.get_uniform("mat_dif").unwrap();
    let spe_uniform = vert_prog.get_uniform("mat_spe").unwrap();

    Uniforms {
        model_matrix: model_uniform,
        camera_matrix: cam_uniform,
        projection_matrix: proj_uniform,
        light_colour: light_uniform,
        material_emission: emi_uniform,
        material_ambient: amb_uniform,
        material_diffuse: dif_uniform,
        material_specular: spe_uniform,
    }
}

pub struct DrawSprites;

impl RenderCommand for DrawSprites {
    type Param = SRes<SpriteBatches>;

    fn render<'w, 'f, 'g>(
        entity: Res<'w, SpriteBatches>,
        pass: &'f mut RenderPass<'g>,
    ) -> Result<(), crate::render::pass::RenderError> {
        let mut camera_matrix = Matrix4::identity();
        camera_matrix.translate(0., 0., -1.0);
        pass.set_vertex_shader(&SPRITE_SHADER, 0)
            .expect("failed to set sprite shader");
        let uniforms = build_uniforms();
        pass.bind_vertex_uniform(uniforms.camera_matrix, &camera_matrix);
        pass.set_attr_info(&VertexAttrs::from_citro3d(Vertex::attr_info()));
        let view_uniform = SPRITE_SHADER.get_uniform("projMtx").unwrap();
        pass.bind_vertex_uniform(view_uniform, &calculate_projections());
        let mat = Material::new(None, Some(Color::rgba(0.5, 0.5, 0.5, 1.0)), None, false);
        mat.set_uniforms(pass, &uniforms);

        for sprite in &entity.batches {
            for s in &sprite.sprites {
                draw_triangle(pass, &s.verts, &uniforms);
            }
        }
        Ok(())
    }
}
