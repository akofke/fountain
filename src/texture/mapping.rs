use crate::{Point2f, Vec2f, SurfaceInteraction, Float};

#[derive(Copy, Clone)]
pub struct TexCoords {
    pub st: Point2f,
    pub dst_dx: Vec2f,
    pub dst_dy: Vec2f,
}

pub trait TextureMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> TexCoords;
}

pub struct UVMapping {
    pub scale_u: Float,
    pub scale_v: Float,
    pub offset_u: Float,
    pub offset_v: Float,
}

impl TextureMapping2D for UVMapping {
    fn map(&self, si: &SurfaceInteraction) -> TexCoords {
        unimplemented!()
//        let dst_dx = Vec2f::new(self.scale_u * si.tex_diffs)
    }
}