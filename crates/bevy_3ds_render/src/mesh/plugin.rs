use bevy::{
    app::Plugin,
    render::{mesh::Mesh, texture::Image, RenderApp},
};

use crate::{draw::AppDrawCommandsExtra, prep_asset::PrepareAssetsPlugin};

use super::draw::MeshDraw;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PrepareAssetsPlugin::<Mesh, Image>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_render_command::<MeshDraw>();
        }
    }
}
