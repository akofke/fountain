use crate::texture::mapping::{TexCoordsMap2D, TexCoords};
use crate::texture::Texture;
use crate::spectrum::Spectrum;
use crate::SurfaceInteraction;

pub struct UVTexture<M: TexCoordsMap2D> {
    mapping: M,
}

impl<M: TexCoordsMap2D> UVTexture<M> {
    pub fn new(mapping: M) -> Self {
        Self { mapping }
    }
}

impl<M: TexCoordsMap2D> Texture for UVTexture<M> {
    type Output = Spectrum;

    fn evaluate(&self, si: &SurfaceInteraction) -> Self::Output {
        let TexCoords { st, .. } = self.mapping.evaluate(si);
        let red = st.x - st.x.floor();
        let green = st.y - st.y.floor();
        Spectrum::from([red, green, 0.0])
    }
}