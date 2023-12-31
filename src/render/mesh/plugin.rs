use bevy::{
    app::Plugin,
    render::{mesh::Mesh, texture::Image},
};

use crate::render::prep_asset::PrepareAssetsPlugin;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PrepareAssetsPlugin::<Mesh, Image>::default());
    }
}
