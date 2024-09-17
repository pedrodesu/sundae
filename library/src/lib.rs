#[no_mangle]
pub extern "C" fn putd(d: i32) {
    println!("{}", d);
}
