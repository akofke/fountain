use crate::aabb::Aabb;

pub trait Primitive {
    fn world_bound(&self) -> Aabb;

    // TODO
}