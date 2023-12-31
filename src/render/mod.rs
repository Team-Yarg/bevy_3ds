use bevy::ecs::schedule::SystemSet;

mod extract;
mod mesh;
mod plugin;
mod prep_asset;

#[derive(SystemSet)]
pub enum RenderSet3ds {
    PrepareAssets,
}
