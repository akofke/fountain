use crate::interaction::SurfaceInteraction;
use std::ops::{Mul, Deref};
use crate::spectrum::Spectrum;
use crate::Float;
use std::sync::Arc;

pub mod mapping;
pub mod uv;
pub mod checkerboard;
pub mod image;

pub trait Texture: Sync + Send {
    type Output;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output;
}

pub trait FloatTexture = Texture<Output = Float>;
pub trait SpectrumTexture = Texture<Output = Spectrum>;

pub type TextureRef<T> = Arc<dyn Texture<Output=T>>;

impl<O, T> Texture for T
    where T: Deref<Target = dyn Texture<Output=O>>,
          T: Sync + Send
{
    type Output = O;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        self.deref().evaluate(si)
    }
}

pub struct ConstantTexture<T: Copy>(pub T);

impl<T: Copy + Sync + Send> Texture for ConstantTexture<T> {
    type Output = T;

    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.0
    }
}

pub struct ScaleTexture<T1, T2>
where
    T1: Texture,
    T2: Texture,
    T1::Output: Mul<T2::Output>
{
    t1: T1,
    t2: T2,
}

impl<T1, T2> Texture for ScaleTexture<T1, T2>
    where
        T1: Texture,
        T2: Texture,
        T1::Output: Mul<T2::Output>
{
    type Output = <T1::Output as Mul<T2::Output>>::Output;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        self.t1.evaluate(si) * self.t2.evaluate(si)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_dyn_texture() {
         let t1: Arc<dyn Texture<Output=_>> = Arc::new(ConstantTexture(3.0));
        let t2: Arc<dyn Texture<Output=_>> = Arc::new(ConstantTexture(2.0));

        let scale = ScaleTexture {t1, t2};
    }
}

