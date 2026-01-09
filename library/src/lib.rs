use std::ffi::CStr;

#[unsafe(no_mangle)]
pub extern "C" fn putd(d: i32)
{
    println!("{d}");
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is safe if `s` points to a valid C string.
pub unsafe extern "C" fn puts(s: *const std::ffi::c_char)
{
    println!("{}", unsafe { CStr::from_ptr(s) }.to_string_lossy());
}
