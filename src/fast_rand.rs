use nalgebra::Vector2;
use crate::Vec3;
use std::cell::UnsafeCell;
use rand_xoshiro::Xoshiro256Plus;
use rand::distributions::{Distribution, Standard};
use rand::{Rng, FromEntropy};

thread_local!(static RNG: UnsafeCell<Xoshiro256Plus> = UnsafeCell::new(Xoshiro256Plus::from_entropy()));

pub fn rand<T>() -> T where Standard: Distribution<T> {
    RNG.with(|rng_cell| {
        unsafe {
            let rng: &mut Xoshiro256Plus = &mut *rng_cell.get();
            rng.gen()
        }
    })
}

pub fn thread_rng() -> &'static mut Xoshiro256Plus {
    RNG.with(|cell| {
        unsafe {
            &mut *cell.get()
        }
    })
}

pub fn random_in_unit_sphere() -> Vec3 {
    let rng = thread_rng();
    loop {
        let p = 2.0 * Vec3::new(rng.gen(), rng.gen(), rng.gen()) - Vec3::repeat(1.0);
        if p.norm_squared() < 1.0 { break p }
    }
}

pub fn random_in_unit_disk() -> Vector2<f32> {
    let rng = thread_rng();
    loop {
        let p: Vector2<f32> = 2.0 * Vector2::<f32>::new(rng.gen(), rng.gen()) - Vector2::repeat(1.0f32);
        if p.norm_squared() < 1.0 { break p }
    }
}
