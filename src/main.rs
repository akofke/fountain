use std::mem::size_of;

mod math;

use math::{Vector3, Vector3f};

fn main() {
    let v1 = Vector3::new(1.0, 2.0, 3.0);
    let mut v2 = Vector3f::new(2.0, 3.0, 4.0);
    v2 += 3.0;
    dbg!(v2);
}
