use crate::texture::Texture;
use crate::texture::mapping::TextureMapping2D;
use crate::SurfaceInteraction;

pub enum AAMethod {
    None, ClosedForm
}

pub struct Checkerboard2DTexture<T1, T2, M: TextureMapping2D>
    where
        T1: Texture,
        T2: Texture<Output=T1::Output>
{
    tex1: T1,
    tex2: T2,
    mapping: M,
    aa_method: AAMethod
}

impl<T1: Texture, T2: Texture, M: TextureMapping2D> Checkerboard2DTexture<T1, T2, M> {
    pub fn new(tex1: T1, tex2: T2, mapping: M) -> Self {
        Self {
            tex1, tex2, mapping, aa_method: AAMethod::None
        }
    }
}

impl<T1: Texture, T2: Texture, M: TextureMapping2D> Texture for Checkerboard2DTexture<T1, T2, M> {
    type Output = T1::Output;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        let st = self.mapping.map(si);
        match self.aa_method {
            None => {

            },

            ClosedForm => {

            }
        }
        unimplemented!()
    }
}
