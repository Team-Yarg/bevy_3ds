use bevy::{app::App, math::Mat4, render::RenderApp};

pub fn without_render_app<R>(app: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    let r_app = app.remove_sub_app(RenderApp);
    let res = f(app);
    if let Some(r_app) = r_app {
        app.insert_sub_app(RenderApp, r_app);
    }
    res
}

pub fn wgpu_projection_to_opengl(projection: Mat4) -> Mat4 {
    /// 3ds screens are actually tilted 90deg left, this corrects that
    /// This is actually just a -90 along Z rotation but its inline so
    /// it can be const
    #[rustfmt::skip]
    const CORRECT_TILT: Mat4 = Mat4::from_cols_array(&[
        0.0, -1.0,  0.0, 0.0,
        1.0,  0.0,  0.0, 0.0,
        0.0,  0.0,  1.0, 0.0,
        0.0,  0.0,  0.0, 1.0,
    ]);
    /// This turns transforms the depth space from vulkan NDS ([0, -1]) to the PICA200
    /// NDS depth ([-1, 0])
    #[rustfmt::skip]
    const WGPU_TO_OPENGL_DEPTH: Mat4 = Mat4::from_cols_array(&[
        1.0,  0.0,  0.0,  0.0,
        0.0, -1.0,  0.0,  0.0,
        0.0,  0.0,  2.0,  0.0,
        0.0,  0.0, -1.0,  1.0,
    ]);

    CORRECT_TILT * WGPU_TO_OPENGL_DEPTH * projection
}
