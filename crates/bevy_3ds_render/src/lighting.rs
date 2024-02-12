use std::{borrow::Borrow, pin::Pin};

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
use citro3d::light::{LightEnv, LightIndex, LightLutId, LutData, LutInput};

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
    pub shadow: bool,
}

fn ensure_all_lights_created(mut lights: Pin<&mut LightEnv>) {
    let to_add = lights.lights().iter().filter(|l| l.is_none()).count();
    for _ in 0..to_add {
        lights.as_mut().create_light();
    }
}

fn prepare_point_lights(lights: Query<&ExtractedPointLight>, gpu: Res<GpuDevice>) {
    let mut gpu_raw = gpu.inst();
    let mut light_env = gpu_raw.light_env_mut();
    light_env
        .as_mut()
        .connect_lut(LightLutId::D0, LutInput::LightNormal, LutData::phong(30.0));
    ensure_all_lights_created(light_env.as_mut());

    for (mut light, l) in light_env
        .as_mut()
        .lights_mut()
        .iter_mut()
        .map(|l| l.as_pin_mut().unwrap())
        .zip(lights.iter().map(Some).chain(std::iter::once(None).cycle()))
    {
        if let Some(l) = l {
            light.as_mut().set_enabled(true);
            light
                .as_mut()
                .set_color(l.color.r(), l.color.g(), l.color.b());
            let pos = l.transform.compute_transform().translation;
            light.as_mut().set_position(pos.into());
            light.as_mut().set_shadow(l.shadow);
        } else {
            light.as_mut().set_enabled(false);
        }
    }
}
