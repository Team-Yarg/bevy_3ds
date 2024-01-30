use bevy::{app::App, math::Mat4, render::RenderApp};

pub fn without_render_app<R>(app: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    let r_app = app.remove_sub_app(RenderApp);
    let res = f(app);
    if let Some(r_app) = r_app {
        app.insert_sub_app(RenderApp, r_app);
    }
    res
}

pub trait ToOpenGl {
    fn to_opengl(&self) -> Self;
}

#[rustfmt::skip]
const WGPU_TO_OPENGL_TRANSFORM: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0,  0.0, 0.0,
    0.0, 1.0,  0.0, 0.0,
    0.0, 0.0, -1.0, 0.0,
    0.0, 0.0,  0.0, 1.0,
]);

impl ToOpenGl for Mat4 {
    fn to_opengl(&self) -> Self {
        todo!()
    }
}
