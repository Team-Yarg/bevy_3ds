use bevy::{
    app::Plugin,
    ecs::schedule::IntoSystemConfigs,
    render::{Render, RenderApp},
};

use bevy_3ds_render::{draw::AppDrawCommandsExtra, RenderSet3ds};

use self::render::SpriteBatches;

mod render;

pub struct SpritesRenderPlugin;

impl Plugin for SpritesRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpriteBatches>()
                .add_render_command::<render::DrawSprites>()
                .add_systems(
                    Render,
                    render::prepare_sprites.in_set(RenderSet3ds::PrepareBindGroups),
                );
        }
    }
}
