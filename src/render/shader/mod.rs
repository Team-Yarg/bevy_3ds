use bevy::{
    app::Plugin,
    asset::{Asset, AssetApp, AssetLoader, AsyncReadExt},
    ecs::system::Resource,
    reflect::TypePath,
};
use citro3d::shader::Entrypoint;

use super::{pipeline::ShaderLib, prep_asset::PrepareAsset};

pub struct PicaShaderPlugin;

impl Plugin for PicaShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<PicaShader>()
            .init_asset_loader::<PicaShaderLoader>();
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct PicaShader(ShaderLib);

impl PicaShader {
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, PicaShaderLoadError> {
        let shader = citro3d::shader::Library::from_bytes(&bytes)
            .map_err(|e| PicaShaderLoadError::ShaderParse(e.to_string()))?;
        Ok(Self(shader))
    }
    pub fn entry_point(&self, index: usize) -> Option<Entrypoint> {
        self.0.get(index)
    }
}

#[derive(Default)]
pub struct PicaShaderLoader;

#[derive(Debug, thiserror::Error)]
pub enum PicaShaderLoadError {
    #[error(transparent)]
    Citro(#[from] citro3d::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse shader: {0}")]
    ShaderParse(String),
}

impl AssetLoader for PicaShaderLoader {
    type Asset = PicaShader;
    type Error = PicaShaderLoadError;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;
            PicaShader::load_from_bytes(&buf)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pica"]
    }
}
