use std::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

use ctru::linear::LinearAllocator;

pub struct LinearBuffer<T>(Box<[T], LinearAllocator>);

impl<T> std::ops::Deref for LinearBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

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
        Self(box_from_copy_slice(items, LinearAllocator))
    }
}
