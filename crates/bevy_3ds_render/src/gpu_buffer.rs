use std::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

use ctru::linear::LinearAllocator;

#[derive(Debug, Clone)]
pub struct LinearBuffer<T>(Vec<T, LinearAllocator>);

impl<T> std::ops::Deref for LinearBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl<T> std::ops::DerefMut for LinearBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

#[allow(unused)]
fn box_from_copy_slice<T: Copy, A: Allocator>(items: &[T], alloc: A) -> Box<[T], A> {
    unsafe {
        let mem: NonNull<[u8]> = alloc.allocate(Layout::for_value(items)).unwrap();
        core::ptr::copy_nonoverlapping(items.as_ptr(), mem.as_ptr() as *mut _, items.len());
        Box::from_raw_in(mem.as_ptr() as *mut _, alloc)
    }
}

impl<T: Copy> LinearBuffer<T> {
    pub fn new(items: &[T]) -> Self
    where
        T: Sized,
    {
        Self(
            /*box_from_copy_slice(items, LinearAllocator)*/ items.to_vec_in(LinearAllocator),
        )
    }

    pub fn with_size(sz: usize, v: T) -> Self
    where
        T: Sized + Clone,
    {
        let mut vs = Vec::with_capacity_in(sz, LinearAllocator);
        vs.resize(sz, v);
        Self(vs)
    }
}
