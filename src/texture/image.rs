use crate::mipmap::{Texel, MIPMap};
use crate::texture::mapping::{TexCoordsMap2D, TexCoords};
use std::sync::Arc;
use crate::texture::Texture;
use crate::spectrum::Spectrum;
use crate::SurfaceInteraction;

pub struct ImageTexture<T, M>
where
    M: TexCoordsMap2D,
    T: Texel,
{
    mapping: M,
    mipmap: Arc<MIPMap<T>>,
}

impl<T: Texel, M: TexCoordsMap2D> ImageTexture<T, M> {
    pub fn new(mapping: M, mipmap: Arc<MIPMap<T>>) -> Self {
        Self {
            mapping,
            mipmap
        }
    }
}

impl<M: TexCoordsMap2D> Texture for ImageTexture<Spectrum, M> {
    type Output = Spectrum;

    // TODO: handle output type different from storage type
    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        let TexCoords { st, dst_dx, dst_dy } = self.mapping.evaluate(si);
        self.mipmap.lookup_trilinear(st, dst_dx, dst_dy)
    }
}

