use bevy::{
    app::{Plugin, PostUpdate},
    asset::AssetApp,
    ecs::schedule::IntoSystemConfigs,
    render::{view::VisibilitySystems, ExtractSchedule, Render, RenderApp},
    sprite::{
        calculate_bounds_2d, extract_sprite_events, extract_sprites, Anchor, ExtractedSprites,
        Mesh2dHandle, Sprite, SpriteAssetEvents, SpriteSystem, TextureAtlas, TextureAtlasSprite,
    },
};

use bevy_3ds_render::{draw::AppDrawCommandsExtra, RenderSet3ds};

use self::render::SpriteBatches;

mod render;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TextureAtlas>()
            .register_asset_reflect::<TextureAtlas>()
            .register_type::<Sprite>()
            .register_type::<TextureAtlasSprite>()
            .register_type::<Anchor>()
            .register_type::<Mesh2dHandle>()
            //.add_plugins((Mesh2dRenderPlugin, ColorMaterialPlugin))
            .add_systems(
                PostUpdate,
                calculate_bounds_2d.in_set(VisibilitySystems::CalculateBounds),
            );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            // standard bevy
            render_app
                .init_resource::<ExtractedSprites>()
                .init_resource::<SpriteAssetEvents>()
                .add_systems(
                    ExtractSchedule,
                    (
                        extract_sprites.in_set(SpriteSystem::ExtractSprites),
                        extract_sprite_events,
                    ),
                );

            // our stuff
            render_app
                .init_resource::<SpriteBatches>()
                .add_render_command::<render::DrawSprites>()
                .add_systems(
                    Render,
                    render::prepare_sprites.in_set(RenderSet3ds::PrepareBindGroups),
                );
        };
    }

    fn name(&self) -> &str {
        // prevent using with standard
        std::any::type_name::<bevy::sprite::SpritePlugin>()
    }
}
