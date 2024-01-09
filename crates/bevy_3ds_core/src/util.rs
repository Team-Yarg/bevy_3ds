use bevy::{app::App, render::RenderApp};

pub fn without_render_app<R>(app: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    let r_app = app.remove_sub_app(RenderApp);
    let res = f(app);
    if let Some(r_app) = r_app {
        app.insert_sub_app(RenderApp, r_app);
    }
    res
}
