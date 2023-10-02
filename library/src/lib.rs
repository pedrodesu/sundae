#[no_mangle]
pub extern "C" fn putd(d: u32) {
    println!("{}", d);
}
