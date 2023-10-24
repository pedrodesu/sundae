// https://stackoverflow.com/questions/63617012/creating-and-linking-static-rust-library-and-link-to-c
#[no_mangle]
pub extern "C" fn putd(d: u32) {
    println!("{}", d);
}
