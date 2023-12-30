use std::sync::{Arc, Mutex};

use std::ops::Deref;

use bevy::{
    app::{App, Plugin, SubApp},
    ecs::{
        schedule::{Schedule, ScheduleLabel},
        system::{Query, Resource},
        world::World,
    },
    prelude::*,
    render::{
        camera::CameraPlugin,
        globals::GlobalsPlugin,
        mesh::{morph::MorphPlugin, MeshPlugin},
        render_resource::ShaderLoader,
        view::{self, ViewPlugin},
        ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
    },
    transform::components::GlobalTransform,
    ui::{
        node_bundles::NodeBundle, BackgroundColor, ExtractedUiNode, ExtractedUiNodes,
        RenderUiSystem,
    },
    DefaultPlugins,
};
use ctru::{
    console::Console,
    services::{apt::Apt, gfx::Gfx},
};

use crate::gpu_buffer::LinearBuffer;

pub struct ExtractCitro3dPlugin;

impl Plugin for ExtractCitro3dPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
