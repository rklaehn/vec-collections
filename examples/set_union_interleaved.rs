extern crate abc;

use abc::ArraySet;

fn main() {
    let a: ArraySet<u32> = (0..10000).step_by(2).collect();
    let b: ArraySet<u32> = (1..10001).step_by(2).collect();
    let t0 = std::time::Instant::now();
    let r = a | b;
    let dt = std::time::Instant::now() - t0;
    println!("Hi! {} {:?}", r.is_empty(), dt);
}