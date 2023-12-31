use bevy::ecs::{schedule::SystemSet, system::Resource};

mod extract;
mod mesh;
pub mod pass;
mod plugin;
mod prep_asset;
mod texture;

pub use plugin::Render3dsPlugin;
pub use prep_asset::RenderAssets;

#[derive(Resource)]
pub struct GpuDevice {
    instance: citro3d::Instance,
}
impl Default for GpuDevice {
    fn default() -> Self {
        Self {
            instance: citro3d::Instance::new().unwrap(),
        }
    }
}
impl GpuDevice {
    pub fn raw(&self) -> &citro3d::Instance {
        &self.instance
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum RenderSet3ds {
    PrepareAssets,
}
