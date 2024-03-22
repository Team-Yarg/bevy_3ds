use bevy::{
    app::{App, Plugin},
    core_pipeline::{
        clear_color::{ClearColor, ClearColorConfig},
        core_2d::Core2dPlugin,
        core_3d::Camera3d,
        prepass::{DepthPrepass, NormalPrepass},
    },
};

pub struct CorePipeline3ds;
impl Plugin for CorePipeline3ds {
    fn build(&self, app: &mut App) {
        app.register_type::<ClearColor>()
            .register_type::<ClearColorConfig>()
            .register_type::<DepthPrepass>()
            .register_type::<NormalPrepass>()
            .init_resource::<ClearColor>()
            .add_plugins((Core2dPlugin, Core3dPlugin));
        #[cfg(feature = "render")]
        {
            app.add_plugins((bevy::render::extract_resource::ExtractResourcePlugin::<
                ClearColor,
            >::default(),));
        }
    }
}

struct Core3dPlugin;

impl Plugin for Core3dPlugin {
    fn build(&self, app: &mut App) {
        use bevy::core_pipeline::core_3d::Camera3dDepthLoadOp;
        app.register_type::<Camera3d>()
            .register_type::<Camera3dDepthLoadOp>();
        #[cfg(feature = "render")]
        {
            app.add_plugins(bevy::render::extract_component::ExtractComponentPlugin::<
                Camera3d,
            >::default());
        }
    }
}
