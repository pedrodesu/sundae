fn swap<A: Copy>(a: &mut A, b: &mut A) {
    let c: A = *a;
    *a = *b;
    *b = c
}

fn main() {
    let mut a: u32 = 10;
    let mut b: u32 = 32;

    swap(&mut a, &mut b);
}
