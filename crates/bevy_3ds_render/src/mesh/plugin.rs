use bevy::{
    app::Plugin,
    asset::Handle,
    ecs::system::{Query, ResMut, Resource},
    math::Mat4,
    pbr::StandardMaterial,
    render::{mesh::Mesh, texture::Image, Extract, ExtractSchedule, RenderApp},
    transform::components::GlobalTransform,
};
use log::debug;

use crate::{draw::AppDrawCommandsExtra, prep_asset::PrepareAssetsPlugin};

use super::draw::MeshDraw;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PrepareAssetsPlugin::<Mesh, Image>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<MeshDraw>()
                .init_resource::<ExtractedMeshes>()
                .add_systems(ExtractSchedule, extract_meshes);
        }
    }
}

pub struct ExtractedMesh {
    pub mesh: Handle<Mesh>,
    pub transform: Mat4,
    pub material: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct ExtractedMeshes {
    pub extracted: Vec<ExtractedMesh>,
}

#[allow(clippy::type_complexity)]
fn extract_meshes(
    mut extracted: ResMut<ExtractedMeshes>,
    query: Extract<Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform)>>,
) {
    extracted.extracted.clear();

    for (mesh_handle, material_handle, transform) in &query {
        debug!("extract: {mesh_handle:?}");

        extracted.extracted.push(ExtractedMesh {
            mesh: mesh_handle.to_owned(),
            transform: transform.compute_matrix(),
            material: material_handle.to_owned(),
        })
    }
}
