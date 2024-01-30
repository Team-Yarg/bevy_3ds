use std::f32::consts::PI;

use bevy::{
    asset::{AssetId, AssetServer},
    ecs::{
        component::Component,
        entity::Entity,
        system::{lifetimeless::SRes, Query, Res, ResMut, Resource},
    },
    math::{Mat4, Quat, Vec2, Vec3, Vec4},
    render::{color::Color, texture::Image, view::ExtractedView},
    sprite::ExtractedSprites,
};
use citro3d::{
    attrib::Register,
    buffer::{self},
    macros::include_shader,
    math::Matrix4,
    texenv::Stage,
    uniform::Index,
};
use lazy_static::lazy_static;
use tracing::debug;

use bevy_3ds_render::{
    gpu_buffer::LinearBuffer,
    pass::{RenderCommand, RenderPass, VboBuffer},
    pipeline::VertexAttrs,
    shader::PicaShader,
    RenderAssets,
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: Vec2,
    //colour: Vec4,
    uv: Vec2,
}

impl Vertex {
    fn attr_info() -> citro3d::attrib::Info {
        let mut info = citro3d::attrib::Info::new();
        info.add_loader(Register::new(0).unwrap(), citro3d::attrib::Format::Float, 2)
            .unwrap();
        info.add_loader(Register::new(1).unwrap(), citro3d::attrib::Format::Float, 2)
            .unwrap();
        info
    }
}

struct SpriteInstance {
    transform: Mat4,
    verts: LinearBuffer<Vertex>,
    #[allow(unused)]
    indexes: LinearBuffer<u32>,
    mat: Material,
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
    images: Res<RenderAssets<Image>>,
    sprites: Res<ExtractedSprites>,
    mut batches: ResMut<SpriteBatches>,
    assets: Res<AssetServer>,
) {
    batches.batches.clear();
    let mut batch_image_id = AssetId::invalid();
    let mut batch_image_dims = Vec2::ZERO;
    for (_id, sprite) in &sprites.sprites {
        if sprite.image_handle_id != batch_image_id {
            if let Some(img) = images.get(sprite.image_handle_id) {
                batch_image_id = sprite.image_handle_id;
                batch_image_dims = Vec2::new(img.width(), img.height());
            } else {
                if sprite.image_handle_id != AssetId::invalid() {
                    let st = assets.load_state(sprite.image_handle_id);
                    debug!("load state of sprite image: {st:#?}");
                }
                log::warn!(
                    "sprite has invalid image handle '{}', skipping this sprite",
                    sprite.image_handle_id
                );
                continue;
            };
        }

        let mut uv_scale = Vec2::ONE;
        if sprite.flip_x {
            uv_scale.x = -1.0;
        }
        if sprite.flip_y {
            uv_scale.y = -1.0;
        }

        let mut uvs = [
            Vec2::new(0., 1.),
            Vec2::new(0., 0.),
            Vec2::new(1.0, 0.0),
            Vec2::new(1., 1.),
        ];

        let mut bounds = batch_image_dims;

        if let Some(uv_r) = sprite.rect {
            let tl = uv_r.min / batch_image_dims;
            let br = uv_r.max / batch_image_dims;
            let tr = Vec2::new(br.x, tl.y);
            let bl = Vec2::new(tl.x, br.y);
            uvs[0] = tl;
            uvs[2] = br;

            uvs[1] = bl;
            uvs[3] = tr;

            bounds = uv_r.size();
        }
        if sprite.flip_x {
            uvs.swap(0, 3);
            uvs.swap(1, 2);
        }
        if sprite.flip_y {
            uvs.swap(0, 1);
            uvs.swap(3, 2);
        }
        if let Some(sz) = sprite.custom_size {
            bounds = sz;
        }

        let transform = sprite.transform.compute_matrix()
            * Mat4::from_scale_rotation_translation(
                bounds.extend(1.),
                Quat::IDENTITY,
                (bounds * (-sprite.anchor - Vec2::splat(0.5))).extend(0.),
            );
        // order is: bl, tl, tr, br
        let verts = [
            Vec2::new(0.0, 1.),
            Vec2::new(0.0, 0.0),
            Vec2::new(1., 0.0),
            Vec2::splat(1.),
        ];

        let verts: [Vertex; 4] = std::array::from_fn(|i| Vertex {
            pos: verts[i],
            uv: uvs[i],
        });
        let verts = LinearBuffer::new(&verts);
        //let colour = sprite.color.as_rgba_f32().into();
        let indexes = LinearBuffer::new(&[0, 1, 2, 0, 2, 1]);
        let batch = SpriteBatch {
            image: sprite.image_handle_id,
            sprites: vec![SpriteInstance {
                verts,
                indexes,
                transform,
                mat: Material::new(Some(sprite.color), None),
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
    colour: Option<Color>,
    ambient: Option<Color>,
}

impl Material {
    pub fn new(colour: Option<Color>, ambient: Option<Color>) -> Self {
        Self { colour, ambient }
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

#[rustfmt::skip]
const WGPU_TO_OPENGL_DEPTH: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0,  0.0, 0.0,
    0.0, 1.0,  0.0, 0.0,
    0.0, 0.0, -1.0, 0.0,
    0.0, 0.0,  0.0, 1.0,
]);

/// 3ds screens are actually tilted 90deg left, this corrects that
#[rustfmt::skip]
const CORRECT_TILT: Mat4 = Mat4::from_cols_array(&[
    0.0, -1.0,  0.0, 0.0,
    1.0, 0.0,  0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0,  0.0, 1.0,
]);

impl RenderCommand for DrawSprites {
    type Param = (
        SRes<SpriteBatches>,
        SRes<RenderAssets<Image>>,
        Query<'static, 'static, &'static ExtractedView>,
    );

    fn render<'w: 'f, 'f>(
        (entity, images, views): (
            Res<'w, SpriteBatches>,
            Res<'w, RenderAssets<Image>>,
            Query<&ExtractedView>,
        ),
        pass: &mut RenderPass<'_, 'f>,
        view_id: Entity,
    ) -> Result<(), bevy_3ds_render::pass::RenderError> {
        let entity = entity.into_inner();
        let images = images.into_inner();
        let view = views.get(view_id).expect("failed to find view for draw");
        pass.set_vertex_shader(&SPRITE_SHADER, 0)
            .expect("failed to set sprite shader");
        let view_proj = WGPU_TO_OPENGL_DEPTH * view.projection * CORRECT_TILT;

        let uniforms = build_uniforms();
        pass.bind_vertex_uniform_bevy(uniforms.camera_matrix, &view.transform.compute_matrix());
        pass.set_attr_info(&VertexAttrs::from_citro3d(Vertex::attr_info()));
        let view_uniform = SPRITE_SHADER.get_uniform("projMtx").unwrap();
        pass.bind_vertex_uniform_bevy(view_uniform, &view_proj);
        //pass.bind_vertex_uniform(view_uniform, &calculate_projections());
        log::debug!("draw sprites, {} batches", entity.batches.len());

        for sprite in &entity.batches {
            let img = images.get(sprite.image);
            let uses_img = img.is_some();
            if let Some(t) = img {
                debug!("bind texture for batch");
                pass.bind_texture(0, t);
            }
            pass.configure_texenv(Stage::new(0).unwrap(), |s0| {
                if uses_img {
                    s0.reset();
                    s0.src(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::Source::Texture0,
                        None,
                        None,
                    )
                    .func(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::CombineFunc::Replace,
                    );
                } else {
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
                }
            });

            for s in &sprite.sprites {
                s.mat.set_uniforms(pass, &uniforms);
                //pass.bind_vertex_uniform(uniforms.model_matrix, &Matrix4::identity());
                pass.bind_vertex_uniform_bevy(uniforms.model_matrix, &s.transform);
                log::debug!("transform: {:#?}", s.transform);

                let mut buf = VboBuffer::new();
                let vbo = buf
                    .add(&s.verts, &Vertex::attr_info())
                    .expect("failed to add vbo data");

                pass.set_attr_info(&VertexAttrs::from_citro3d(Vertex::attr_info()));
                pass.draw(buffer::Primitive::TriangleFan, vbo);

                /*let mut buf = VboBuffer::new();
                let verts = [Vertex {
                    pos: Vec2::new(0., 0.),
                    uv: Vec2::new(0., 0.),
                }];
                let verts = LinearBuffer::new(&verts);
                let vbo = buf
                    .add(&verts, &Vertex::attr_info())
                    .expect("failed to add vbo data");
                pass.draw(buffer::Primitive::TriangleFan, vbo);*/
            }
        }
        Ok(())
    }
}
