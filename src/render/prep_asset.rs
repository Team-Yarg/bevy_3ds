use std::marker::PhantomData;

use bevy::{
    app::Plugin,
    asset::{Asset, AssetId},
    ecs::system::{ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem},
    render::{
        render_asset::{
            prepare_assets, ExtractedAssets, PrepareAssetError, PrepareNextFrameAssets,
            RenderAsset, RenderAssetDependency, RenderAssetPlugin, RenderAssets,
        },
        RenderApp,
    },
};

use super::RenderSet3ds;

pub struct PrepareAssetsPlugin<R: RenderAsset, After: RenderAssetDependency + 'static = ()> {
    p: PhantomData<fn() -> (R, After)>,
}

impl<R: RenderAsset, After: RenderAssetDependency + 'static> Default
    for PrepareAssetsPlugin<R, After>
{
    fn default() -> Self {
        Self {
            p: Default::default(),
        }
    }
}

impl<R: RenderAsset, After: RenderAssetDependency + 'static> Plugin
    for PrepareAssetsPlugin<R, After>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(RenderAssetPlugin::<R, After>::default());
        if let Ok(render) = app.get_sub_app_mut(RenderApp) {
            After::register_system(
                render,
                prepare_assets::<R>.in_set(RenderSet3ds::PrepareAssets),
            );
        }
    }
}

/// 3ds gpu specific asset preparation trait
pub trait PrepareAsset: RenderAsset {
    type Param: SystemParam;

    fn prepare_asset_3ds(
        extracted: <Self as RenderAsset>::ExtractedAsset,
        param: &mut SystemParamItem<<Self as PrepareAsset>::Param>,
    ) -> Result<
        <Self as RenderAsset>::PreparedAsset,
        PrepareAssetError<<Self as RenderAsset>::ExtractedAsset>,
    >;
}

/*
#[derive(Resource)]
struct PrepNextFrameAssets<R: RenderAsset> {
    assets: Vec<(AssetId<R>, R::ExtractedAsset)>,
}


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
