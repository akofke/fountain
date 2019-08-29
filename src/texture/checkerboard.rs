use crate::texture::{Texture, ConstantTexture};
use crate::texture::mapping::{TextureMapping2D, TexCoords, UVMapping};
use crate::SurfaceInteraction;
use crate::spectrum::Spectrum;

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

impl<T1, T2, M> Checkerboard2DTexture<T1, T2, M>
    where
        M: TextureMapping2D,
        T1: Texture,
        T2: Texture<Output=T1::Output>
{
    pub fn new(tex1: T1, tex2: T2, mapping: M) -> Self {
        Self {
            tex1, tex2, mapping, aa_method: AAMethod::None
        }
    }
}

impl Default for Checkerboard2DTexture<ConstantTexture<Spectrum>, ConstantTexture<Spectrum>, UVMapping> {
    fn default() -> Self {
        Self::new(ConstantTexture(Spectrum::new(0.0)), ConstantTexture(Spectrum::new(1.0)), UVMapping::new(10.0, 10.0, 0.0, 0.0))
    }
}

impl<T1, T2, M> Texture for Checkerboard2DTexture<T1, T2, M>
    where
        M: TextureMapping2D,
        T1: Texture,
        T2: Texture<Output=T1::Output>
{
    type Output = T1::Output;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        let TexCoords { st, dst_dx, dst_dy } = self.mapping.map(si);
        match self.aa_method {
            AAMethod::None => {
                if (st[0].floor() as i32 + st[1].floor() as i32) % 2 == 0 {
                    self.tex1.evaluate(si)
                } else {
                    self.tex2.evaluate(si)
                }
            },

            AAMethod::ClosedForm => {
                unimplemented!()
            }
        }
    }
}
