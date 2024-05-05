#![feature(allocator_api)]

use self::pipeline::VertexAttrs;
use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        schedule::SystemSet,
        system::{Commands, Query, Resource},
    },
    hierarchy::Children,
    render::{
        extract_component::ExtractComponent, render_resource::PrimitiveTopology, texture::Image,
        view::ExtractedView,
    },
};
use citro3d::{buffer::Primitive, Instance};
use ctru::services::gfx::Side;
use frame::Citro3dFrame;
pub use plugin::Render3dsPlugin;
pub use prep_asset::RenderAssets;
use std::{
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
};

pub use texture::GpuImage;
pub mod draw;
mod extract;
mod frame;
pub mod gpu_buffer;
pub mod lighting;
pub mod material;
pub mod materials;
pub mod mesh;
pub mod pass;
pub mod pipeline;
pub mod plugin;
mod prep_asset;
pub mod shader;
pub mod texture;
pub mod vertattr;

pub use citro3d;

pub struct GfxInstance(pub ctru::services::gfx::Gfx);

pub type StereoFunction = fn(&ExtractedView) -> Option<(ExtractedView, ExtractedView)>;

#[derive(Component, Clone, Copy, ExtractComponent)]
#[non_exhaustive]
pub enum On3dsScreen {
    Bottom,
    Top(Option<StereoFunction>),
}

impl Default for On3dsScreen {
    fn default() -> Self {
        Self::Top(None)
    }
}

impl On3dsScreen {
    pub fn to_target_index(&self, side: Option<Side>) -> usize {
        match (side, self) {
            (_, Self::Bottom) => 0,
            (None, Self::Top(_)) => 1,
            (Some(Side::Left), Self::Top(_)) => 1,
            (Some(Side::Right), Self::Top(_)) => 2,
        }
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, PartialEq, Eq, Debug)]
pub struct CameraID(u32);

impl Default for CameraID {
    fn default() -> Self {
        Self(0)
    }
}

impl From<u32> for CameraID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl CameraID {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Component, Clone, ExtractComponent, Debug)]
pub enum RenderOn {
    Specific(Vec<CameraID>),
    Only(CameraID),
    Except(CameraID),
}

impl Default for RenderOn {
    fn default() -> Self {
        Self::Only(CameraID::default())
    }
}

impl RenderOn {
    pub fn should_render_in(&self, cam: CameraID) -> bool {
        match self {
            RenderOn::Specific(ids) => ids.contains(&cam),
            RenderOn::Only(id) => id == &cam,
            RenderOn::Except(id) => id != &cam,
        }
    }

    pub fn make_pending(self) -> PendingRenderOn {
        PendingRenderOn(self)
    }
}

#[derive(Component)]
pub struct PendingRenderOn(RenderOn);

fn set_render_on(
    commands: &mut Commands,
    children: &Children,
    pending_render_on: &PendingRenderOn,
    scene_elements: &Query<&Children, Without<PendingRenderOn>>,
) {
    for child in children {
        if let Ok(children) = scene_elements.get(*child) {
            set_render_on(commands, children, pending_render_on, scene_elements)
        }

        commands.entity(*child).insert(pending_render_on.0.clone());
    }
}

pub fn pending_render_system(
    mut commands: Commands,
    scenes: Query<(Entity, &Children, &PendingRenderOn)>,
    scene_elements: Query<&Children, Without<PendingRenderOn>>,
) {
    for (entity, children, pending_render_on) in &scenes {
        set_render_on(&mut commands, children, pending_render_on, &scene_elements);
        commands.entity(entity).remove::<PendingRenderOn>();
    }
}

impl Default for GfxInstance {
    fn default() -> Self {
        Self(
            ctru::services::gfx::Gfx::with_formats_shared(
                ctru::services::gspgpu::FramebufferFormat::Rgba8,
                ctru::services::gspgpu::FramebufferFormat::Rgba8,
            )
            .expect("failed to init gpu"),
        )
    }
}

#[derive(Resource)]
pub struct GpuDevice {
    instance: Mutex<citro3d::Instance>,
}
impl Default for GpuDevice {
    fn default() -> Self {
        let instance = Mutex::new(citro3d::Instance::new().unwrap());
        unsafe {citro3d_sys::C3D_AlphaTest(true, ctru_sys::GPU_GREATER, 0)};
        Self { instance }
    }
}
impl GpuDevice {
    fn inst(&self) -> MutexGuard<Instance> {
        self.instance.lock().unwrap()
    }
    pub fn start_new_frame(&self) -> Citro3dFrame<'_> {
        Citro3dFrame::new(self)
    }

    /// set the shader program to use for subsequent draw calls
    ///
    /// # Safety
    /// If the shader is drop'd before the frame ends then it will result in a use-after-free
    pub unsafe fn set_shader(&self, shader: Pin<Arc<citro3d::shader::Program>>) {
        self.inst().bind_program(shader);
    }

    /// Set the attribute info for subsequent draw calls
    pub fn set_attr_info(&self, attr: &VertexAttrs) {
        self.inst().set_attr_info(&attr.0);
    }

    /// Draw vertexes
    ///
    /// # Safety
    /// If `verts` goes out of scope before the frame ends there will be a use-after-free by the GPU
    pub unsafe fn draw(&self, prim: Primitive, verts: citro3d::buffer::Slice) {
        self.inst().draw_arrays(prim, verts);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum RenderSet3ds {
    PrepareAssets,
    Prepare,
    PrepareBindGroups,
}

pub fn bevy_topology_to_citro(topology: PrimitiveTopology) -> Option<Primitive> {
    match topology {
        PrimitiveTopology::TriangleList => Some(Primitive::Triangles),
        PrimitiveTopology::TriangleStrip => Some(Primitive::TriangleStrip),
        _ => None,
    }
}
