use crate::Float;

pub trait Sampler {
    fn get_1d(&self) -> Float;
}