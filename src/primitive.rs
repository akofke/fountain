use crate::geometry::bounds::Bounds3f;

pub trait Primitive {
    fn world_bound(&self) -> Bounds3f;

    // TODO
}