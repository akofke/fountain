use crate::Vec3;
use std::f32;

/// Axis-aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3
}

pub trait Bounded {
    fn aabb(&self) -> Aabb;
}

impl Aabb {
    pub fn with_bounds(min: Vec3, max: Vec3) -> Self {
        Self {min, max}
    }

    pub fn empty() -> Self {
        Self::with_bounds(Vec3::repeat(f32::INFINITY), Vec3::repeat(f32::NEG_INFINITY))
    }

    pub fn join(&self, other: &Aabb) -> Self {
        Self::with_bounds(
            Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            )

        )
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn centroid(&self) -> Vec3 {
        self.min + (self.size() / 2.0)
    }
}