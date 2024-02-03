use bevy::{
    asset::{AssetId, AssetServer},
    ecs::{
        component::Component,
        entity::Entity,
        system::{lifetimeless::SRes, Query, Res, ResMut, Resource},
    },
    math::{Mat4, Quat, Vec2},
    render::{texture::Image, view::ExtractedView},
    sprite::ExtractedSprites,
};
use citro3d::{
    buffer::{self},
    macros::include_shader,
    texenv::Stage,
};
use lazy_static::lazy_static;
use tracing::debug;

use bevy_3ds_render::{
    gpu_buffer::LinearBuffer,
    material::{Material, Uniforms},
    pass::{RenderCommand, RenderPass, VboBuffer},
    pipeline::VertexAttrs,
    shader::PicaShader,
    vertattr::{VertAttrBuilder, VertAttrs},
    RenderAssets,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, VertAttrBuilder)]
struct Vertex {
    pos: Vec2,
    //colour: Vec4,
    uv: Vec2,
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

pub struct DrawSprites;

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
        let uniforms = Uniforms::build(&SPRITE_SHADER);
        uniforms.bind_views(pass, view);
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
                pass.bind_vertex_uniform(uniforms.model_matrix, s.transform);
                let mut buf = VboBuffer::new();
                let vbo = buf
                    .add(&s.verts, &Vertex::vert_attrs())
                    .expect("failed to add vbo data");

                pass.set_attr_info(&VertexAttrs::from_citro3d(Vertex::vert_attrs()));
                pass.draw(buffer::Primitive::TriangleFan, vbo);
            }
        }
        Ok(())
    }
}
