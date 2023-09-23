#[no_mangle]
pub extern "C" fn putd(d: std::ffi::c_uint) {
    println!("{}", d);
}
