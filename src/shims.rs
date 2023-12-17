extern crate shim_3ds;
use libc::{c_char, c_int, c_void};

use core::ptr::null_mut;

#[no_mangle]
pub unsafe extern "C" fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn dlsym(
    handle: *mut c_void,
    symbol: *const c_char
) -> *mut c_void {
    unimplemented!();

}
#[no_mangle]
pub unsafe extern "C" fn dlerror() -> *mut c_char {
    unimplemented!();

}
#[no_mangle]
pub unsafe extern "C" fn dlclose(handle: *mut c_void) -> c_int {
    unimplemented!();
}