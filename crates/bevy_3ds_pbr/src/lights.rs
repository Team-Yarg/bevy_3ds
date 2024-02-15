use bevy::{
    ecs::system::{Commands, Query},
    pbr::PointLight,
    render::Extract,
    transform::components::GlobalTransform,
};
use bevy_3ds_render::lighting::ExtractedPointLight;

pub fn extract_point_lights(
    mut cmds: Commands,
    lights: Extract<Query<(&PointLight, &GlobalTransform)>>,
) {
    for (light, transform) in &lights {
        cmds.spawn(ExtractedPointLight {
            color: light.color,
            transform: transform.to_owned(),
            radius: light.radius,
            intensity: light.intensity,
            shadow: light.shadows_enabled,
            range: light.range,
        });
    }
}
