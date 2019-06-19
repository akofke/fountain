use crate::{Vec3f, Vec2f};
use std::cell::UnsafeCell;
use rand_xoshiro::Xoshiro256Plus;
use rand::distributions::{Distribution, Standard};
use rand::{Rng, FromEntropy, SeedableRng};
use cgmath::InnerSpace;

thread_local!(static RNG: UnsafeCell<Xoshiro256Plus> = UnsafeCell::new(Xoshiro256Plus::from_entropy()));

pub fn rand<T>() -> T where Standard: Distribution<T> {
    RNG.with(|rng_cell| {
        unsafe {
            let rng: &mut Xoshiro256Plus = &mut *rng_cell.get();
            rng.gen()
        }
    })
}

pub fn set_seed(seed: u64) {
    RNG.with(|cell| {
        let rng: *mut Xoshiro256Plus = cell.get();
        unsafe { rng.write(Xoshiro256Plus::seed_from_u64(seed)); }
    })
}

pub fn thread_rng() -> &'static mut Xoshiro256Plus {
    RNG.with(|cell| {
        unsafe {
            &mut *cell.get()
        }
    })
}

pub fn random_in_unit_sphere() -> Vec3f {
    let rng = thread_rng();
    loop {
        let p = 2.0 * Vec3f::new(rng.gen(), rng.gen(), rng.gen()) - Vec3f::new(1.0, 1.0, 1.0);
        if p.magnitude2() < 1.0 { break p }
    }
}

pub fn random_in_unit_disk() -> Vec2f {
    let rng = thread_rng();
    loop {
        let p: Vec2f = 2.0 * Vec2f::new(rng.gen(), rng.gen()) - Vec2f::new(1.0, 1.0);
        if p.magnitude2() < 1.0 { break p }
    }
}
