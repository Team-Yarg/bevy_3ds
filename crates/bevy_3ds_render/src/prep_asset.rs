use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use bevy::{
    app::Plugin,
    asset::{Asset, AssetEvent, AssetId, Assets},
    ecs::{
        event::EventReader,
        system::{
            Commands, Res, ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem,
        },
    },
    prelude::*,
    render::{
        render_asset::{
            PrepareAssetError, PrepareNextFrameAssets, RenderAsset, RenderAssetDependency,
            RenderAssetPlugin,
        },
        Extract, ExtractSchedule, RenderApp,
    },
};

use super::RenderSet3ds;

pub struct PrepareAssetsPlugin<R: PrepareAsset, After: RenderAssetDependency + 'static = ()> {
    p: PhantomData<fn() -> (R, After)>,
}

impl<R: PrepareAsset, After: RenderAssetDependency + 'static> Default
    for PrepareAssetsPlugin<R, After>
{
    fn default() -> Self {
        Self {
            p: Default::default(),
        }
    }
}

impl<R: PrepareAsset, After: RenderAssetDependency + 'static> Plugin
    for PrepareAssetsPlugin<R, After>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Ok(render) = app.get_sub_app_mut(RenderApp) {
            render
                .init_resource::<ExtractedAssets<R>>()
                .init_resource::<RenderAssets<R>>()
                .init_resource::<PrepNextFrameAssets<R>>()
                .add_systems(ExtractSchedule, extract_render_asset::<R>);

            After::register_system(
                render,
                prepare_assets::<R>.in_set(RenderSet3ds::PrepareAssets),
            );
        }
    }
}

/// 3ds gpu specific asset preparation trait
pub trait PrepareAsset: RenderAsset {
    type PreparedAsset: Send + Sync + 'static;
    type Param: SystemParam;

    fn prepare_asset_3ds(
        extracted: <Self as RenderAsset>::ExtractedAsset,
        param: &mut SystemParamItem<<Self as PrepareAsset>::Param>,
    ) -> Result<
        <Self as PrepareAsset>::PreparedAsset,
        PrepareAssetError<<Self as RenderAsset>::ExtractedAsset>,
    >;
}

#[derive(Resource)]
pub struct RenderAssets<R: PrepareAsset>(HashMap<AssetId<R>, <R as PrepareAsset>::PreparedAsset>);

impl<R: PrepareAsset> RenderAssets<R> {
    pub fn get(&self, id: impl Into<AssetId<R>>) -> Option<&<R as PrepareAsset>::PreparedAsset> {
        self.0.get(&id.into())
    }

    pub fn insert(&mut self, id: impl Into<AssetId<R>>, asset: <R as PrepareAsset>::PreparedAsset) {
        self.0.insert(id.into(), asset);
    }

    pub fn remove(
        &mut self,
        id: impl Into<AssetId<R>>,
    ) -> Option<<R as PrepareAsset>::PreparedAsset> {
        self.0.remove(&id.into())
    }
    pub fn iter(&self) -> impl Iterator<Item = (AssetId<R>, &<R as PrepareAsset>::PreparedAsset)> {
        self.0.iter().map(|(k, v)| (*k, v))
    }
}

impl<R: PrepareAsset> Default for RenderAssets<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource)]
struct ExtractedAssets<R: PrepareAsset> {
    extracted: Vec<(AssetId<R>, R::ExtractedAsset)>,
    removed: Vec<AssetId<R>>,
}

impl<R: PrepareAsset> Default for ExtractedAssets<R> {
    fn default() -> Self {
        Self {
            extracted: Default::default(),
            removed: Default::default(),
        }
    }
}

#[derive(Resource)]
struct PrepNextFrameAssets<R: PrepareAsset> {
    assets: Vec<(AssetId<R>, R::ExtractedAsset)>,
}
impl<R: PrepareAsset> Default for PrepNextFrameAssets<R> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

fn extract_render_asset<A: PrepareAsset>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<A>>>,
    assets: Extract<Res<Assets<A>>>,
) {
    trace!("extract render assets");
    let mut changed_assets = HashSet::<AssetId<A>>::new();
    let mut removed = Vec::new();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                debug!("added asset '{id}'");
                changed_assets.insert(*id);
            }
            AssetEvent::Removed { id } => {
                debug!("removed asset '{id}'");
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
            extracted_assets.push((id, asset.extract_asset()));
        }
    }

    commands.insert_resource(ExtractedAssets {
        extracted: extracted_assets,
        removed,
    });
}
fn prepare_assets<R: PrepareAsset>(
    mut extracted_assets: ResMut<ExtractedAssets<R>>,
    mut render_assets: ResMut<RenderAssets<R>>,
    mut prepare_next_frame: ResMut<PrepNextFrameAssets<R>>,
    param: StaticSystemParam<<R as PrepareAsset>::Param>,
) {
    let mut param = param.into_inner();
    let queued_assets = std::mem::take(&mut prepare_next_frame.assets);
    for (id, extracted_asset) in queued_assets {
        match R::prepare_asset_3ds(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                debug!("add asset after retrying: {}", id);
                render_assets.insert(id, prepared_asset);
            }
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                prepare_next_frame.assets.push((id, extracted_asset));
            }
        }
    }

    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_assets.remove(removed);
    }

    for (id, extracted_asset) in std::mem::take(&mut extracted_assets.extracted) {
        match R::prepare_asset_3ds(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                debug!("add asset from extract: {}", id);
                render_assets.insert(id, prepared_asset);
            }
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                debug!("failed to load asset {}, trying again next frame", id);
                prepare_next_frame.assets.push((id, extracted_asset));
            }
        }
    }
}
/*

fn prepare_assets<R: RenderAsset>(
    mut extracted: ResMut<ExtractedAssets<R>>,
    mut preped: ResMut<RenderAssets<R>>,
    mut next_frame: ResMut<PrepNextFrameAssets<R>>,
    param: StaticSystemParam<R::Param>,
) {
    let mut try_prepare_assets = move |todo| {
        let mut param = param.into_inner();
        for (id, asset) in todo {
            match R::prepare_asset_3ds(asset, &mut param) {
                Ok(prepped) => {
                    preped.insert(id, prepped);
                }
                Err(PrepareAssetError::RetryNextUpdate(extracted)) => {
                    next_frame.assets.push((id, extracted));
                }
            }
        }
    };
    let todo = std::mem::take(&mut next_frame.assets);
    try_prepare_assets(todo);

    let mut param = param.into_inner();
    let todo = std::mem::take(&mut next_frame.assets);
    for (id, asset) in todo {
        match R::prepare_asset_3ds(asset, &mut param) {
            Ok(prepped) => {
                preped.insert(id, prepped);
            }
            Err(PrepareAssetError::RetryNextUpdate(extracted)) => {
                next_frame.assets.push((id, extracted));
            }
        }
    }
}
*/
