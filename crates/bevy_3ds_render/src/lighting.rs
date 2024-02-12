use std::borrow::Borrow;

use bevy::{
    app::Plugin,
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    render::{color::Color, Extract, ExtractSchedule, Render, RenderApp},
    transform::components::GlobalTransform,
};
use citro3d::light::{LightIndex, LightLutId, LutData, LutInput};

use crate::{GpuDevice, RenderSet3ds};

pub struct LightingRenderPlugin;
impl Plugin for LightingRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Render, prepare_point_lights.in_set(RenderSet3ds::Prepare));
        }
    }
}

#[derive(Component)]
pub struct ExtractedPointLight {
    pub transform: GlobalTransform,
    pub radius: f32,
    pub intensity: f32,
    pub color: Color,
}

fn prepare_point_lights(lights: Query<&ExtractedPointLight>, gpu: Res<GpuDevice>) {
    let mut gpu_raw = gpu.inst();
    let light_env = gpu_raw.light_env_mut();
    light_env.connect_lut(LightLutId::D0, LutInput::LightNormal, LutData::phong(30.0));

    for (i, l) in lights.iter().enumerate() {
        let light = if let Some(l) = light_env.light_mut(LightIndex::new(i)) {
            l
        } else {
            let l = light_env.create_light().unwrap();
            light_env.light_mut(l).unwrap()
        };
        light.set_color(l.color.r(), l.color.g(), l.color.b());
        let pos = l.transform.compute_transform().translation;
        light.set_position(pos.into());
        unsafe {
            citro3d_sys::C3D_LightShadowEnable(light as *mut _ as *mut _, true);
        }
    }
}
