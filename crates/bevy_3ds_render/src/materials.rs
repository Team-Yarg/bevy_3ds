use std::collections::{HashMap, HashSet};

use bevy::{
    app::Plugin,
    asset::{AssetEvent, AssetId, Assets},
    ecs::{
        event::EventReader,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource},
    },
    pbr::StandardMaterial,
    render::{Extract, ExtractSchedule, Render, RenderApp},
};

use crate::RenderSet3ds;

pub struct StandardMaterialPlugin;
impl Plugin for StandardMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Ok(render) = app.get_sub_app_mut(RenderApp) {
            render
                .init_resource::<ExtractedMats>()
                .init_resource::<RenderMaterials>()
                .add_systems(ExtractSchedule, extract_mats)
                .add_systems(Render, prepare_mats.in_set(RenderSet3ds::PrepareAssets));
        }
    }
}

type RenderMat = StandardMaterial;

#[derive(Resource, Default)]
pub struct RenderMaterials(HashMap<AssetId<StandardMaterial>, RenderMat>);

impl RenderMaterials {
    pub fn get(&self, id: impl Into<AssetId<StandardMaterial>>) -> Option<&RenderMat> {
        self.0.get(&id.into())
    }

    pub fn insert(&mut self, id: impl Into<AssetId<StandardMaterial>>, asset: RenderMat) {
        self.0.insert(id.into(), asset);
    }

    pub fn remove(&mut self, id: impl Into<AssetId<StandardMaterial>>) -> Option<RenderMat> {
        self.0.remove(&id.into())
    }
    pub fn iter(&self) -> impl Iterator<Item = (AssetId<StandardMaterial>, &RenderMat)> {
        self.0.iter().map(|(k, v)| (*k, v))
    }
}

#[derive(Resource, Default)]
struct ExtractedMats {
    extracted: Vec<(AssetId<StandardMaterial>, RenderMat)>,
    removed: Vec<AssetId<StandardMaterial>>,
}

fn extract_mats(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<StandardMaterial>>>,
    assets: Extract<Res<Assets<StandardMaterial>>>,
) {
    let mut changed_assets = HashSet::<AssetId<StandardMaterial>>::new();
    let mut removed = Vec::new();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed_assets.insert(*id);
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(id);
                removed.push(*id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {
                // TODO: handle this
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for id in changed_assets.drain() {
        if let Some(asset) = assets.get(id) {
            extracted_assets.push((id, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedMats {
        extracted: extracted_assets,
        removed,
    });
}
fn prepare_mats(
    mut extracted_assets: ResMut<ExtractedMats>,
    mut render_assets: ResMut<RenderMaterials>,
) {
    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_assets.remove(removed);
    }

    for (id, extracted_asset) in std::mem::take(&mut extracted_assets.extracted) {
        render_assets.insert(id, extracted_asset);
    }
}
