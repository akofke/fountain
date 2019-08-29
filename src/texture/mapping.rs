use crate::{Point2f, Vec2f, SurfaceInteraction, Float};

#[derive(Copy, Clone)]
pub struct TexCoords {
    pub st: Point2f,
    pub dst_dx: Vec2f,
    pub dst_dy: Vec2f,
}

pub trait TextureMapping2D: Sync + Send {
    fn map(&self, si: &SurfaceInteraction) -> TexCoords;
}

pub struct UVMapping {
    pub scale_u: Float,
    pub scale_v: Float,
    pub offset_u: Float,
    pub offset_v: Float,
}

impl UVMapping {
    pub fn new(scale_u: Float, scale_v: Float, offset_u: Float, offset_v: Float) -> Self {
        Self {
            scale_u, scale_v, offset_u, offset_v
        }
    }
}

impl Default for UVMapping {
    fn default() -> Self {
        Self {
            scale_u: 1.0,
            scale_v: 1.0,
            offset_u: 0.0,
            offset_v: 0.0
        }
    }
}

impl TextureMapping2D for UVMapping {
    fn map(&self, si: &SurfaceInteraction) -> TexCoords {
        let dst_dx = Vec2f::new(self.scale_u * si.tex_diffs.dudx, self.scale_v * si.tex_diffs.dvdx);
        let dst_dy = Vec2f::new(self.scale_u * si.tex_diffs.dudy, self.scale_v * si.tex_diffs.dvdy);

        let st = Point2f::new(
            self.scale_u * si.uv.x + self.offset_u,
            self.scale_v * si.uv.y + self.offset_v
        );
        TexCoords {
            st, dst_dx, dst_dy
        }
    }
}