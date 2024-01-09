use bevy::{
    app::Plugin,
    core_pipeline::{core_2d::Camera2d, core_3d::Camera3d},
    ecs::schedule::IntoSystemConfigs,
    render::{ExtractSchedule, RenderApp},
    ui::{
        extract_atlas_uinodes, extract_default_ui_camera_view, extract_text_uinodes,
        extract_uinode_borders, extract_uinode_outlines, extract_uinodes, ExtractedUiNodes,
        RenderUiSystem,
    },
};
use bevy_3ds_core::util;

mod render;

#[derive(Default)]
pub struct UiPlugin {
    inner: bevy::ui::UiPlugin,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        util::without_render_app(app, |app| {
            self.inner.build(app);
        });
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<ExtractedUiNodes>().add_systems(
                ExtractSchedule,
                (
                    extract_default_ui_camera_view::<Camera2d>,
                    extract_default_ui_camera_view::<Camera3d>,
                    extract_uinodes.in_set(RenderUiSystem::ExtractNode),
                    extract_atlas_uinodes
                        .in_set(RenderUiSystem::ExtractAtlasNode)
                        .after(RenderUiSystem::ExtractNode),
                    extract_uinode_borders.after(RenderUiSystem::ExtractAtlasNode),
                    // note this is gated behind a text feature
                    extract_text_uinodes.after(RenderUiSystem::ExtractAtlasNode),
                    extract_uinode_outlines.after(RenderUiSystem::ExtractAtlasNode),
                ),
            );
        }
    }

    fn ready(&self, _app: &bevy::prelude::App) -> bool {
        true
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        util::without_render_app(app, |app| {
            self.inner.finish(app);
        })
    }

    fn cleanup(&self, app: &mut bevy::prelude::App) {
        self.inner.cleanup(app)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}
