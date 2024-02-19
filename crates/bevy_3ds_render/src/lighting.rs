use std::{f32::consts::PI, pin::Pin};

use bevy::{
    app::Plugin,
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
        system::{Query, Res, ResMut, Resource},
    },
    math::Vec3,
    render::{color::Color, Render, RenderApp},
    transform::components::GlobalTransform,
};
use citro3d::light::{LightEnv, LightIndex, LightLutDistAtten};

use crate::{GpuDevice, RenderSet3ds};

pub struct LightingRenderPlugin;
impl Plugin for LightingRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(Render, prepare_point_lights.in_set(RenderSet3ds::Prepare))
                .init_resource::<GpuLights>();
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
    pub range: f32,
}

fn ensure_all_lights_created(mut lights: Pin<&mut LightEnv>, max: usize) {
    let to_add = lights.lights().iter().filter(|l| l.is_none()).count();
    for _ in 0..to_add.min(max) {
        lights.as_mut().create_light();
    }
}

pub struct GpuLight {
    pub index: LightIndex,
    pub global_pos: Vec3,
}

#[derive(Resource, Default)]
pub struct GpuLights {
    pub lights: Vec<GpuLight>,
}

fn prepare_point_lights(
    lights: Query<&ExtractedPointLight>,
    gpu: Res<GpuDevice>,
    mut gpu_lights: ResMut<GpuLights>,
) {
    let mut gpu_raw = gpu.inst();
    let mut light_env = gpu_raw.light_env_mut();
    let nb_lights = lights.iter().len();
    ensure_all_lights_created(light_env.as_mut(), nb_lights);
    gpu_lights.lights.clear();

    for (i, (light, l)) in light_env
        .as_mut()
        .lights_mut()
        .iter_mut()
        .map(|l| l.as_pin_mut())
        .zip(lights.iter().map(Some).chain(std::iter::once(None).cycle()))
        .enumerate()
    {
        if let Some(l) = l {
            let mut light = light.unwrap();
            light.as_mut().set_enabled(true);
            light
                .as_mut()
                .set_color(l.color.r(), l.color.g(), l.color.b());
            let pos = l.transform.compute_transform().translation;
            light.as_mut().set_shadow(l.shadow);
            light
                .as_mut()
                .set_distance_attenutation(Some(LightLutDistAtten::new(
                    l.radius..l.range,
                    |dist| (l.intensity / (4.0 * PI * dist * dist)).min(1.0),
                )));
            gpu_lights.lights.push(GpuLight {
                index: LightIndex::new(i),
                global_pos: pos,
            });
        } else if let Some(mut light) = light {
            light.as_mut().set_enabled(false);
        }
    }
}
