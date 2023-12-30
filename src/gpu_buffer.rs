use ctru::linear::LinearAllocator;

pub type LinearBuffer<T> = Vec<T, LinearAllocator>;
