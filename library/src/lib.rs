use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn println(str: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(str) };

    println!("{}\n", c_str.to_string_lossy());
}
