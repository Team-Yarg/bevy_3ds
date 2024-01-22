use std::marker::PhantomData;

use crate::GpuDevice;

/// Manages the lifetime of a "frame" in citro3d
///
/// citro3d uses frames to batch draw calls, you begin a new frame to begin batching calls
/// and end the frame when you want to actually execute.
///
/// This struct encapsulates this as a RAII wrapper and allows enforcing lifetime bounds on
/// resources bound to the frame
///
/// It is additionally constrained by the lifetime `'g` of the gpu instance to ensure it cannot
/// outlive it
pub struct Citro3dFrame<'g>(PhantomData<&'g ()>);

impl<'g> Citro3dFrame<'g> {
    pub fn new(_gpu: &'g GpuDevice) -> Self {
        unsafe {
            citro3d_sys::C3D_FrameBegin(citro3d_sys::C3D_FRAME_SYNCDRAW.try_into().unwrap());
        }
        Self(PhantomData)
    }
}
impl<'g> Drop for Citro3dFrame<'g> {
    fn drop(&mut self) {
        unsafe {
            citro3d_sys::C3D_FrameEnd(0);
        }
    }
}
