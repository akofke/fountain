use crate::interaction::SurfaceInteraction;

pub trait Texture<T>: Sync + Send {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

pub struct ConstantTexture<T: Copy>(pub T);

impl<T: Copy + Sync + Send> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.0
    }
}