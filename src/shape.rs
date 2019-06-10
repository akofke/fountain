use crate::geometry::bounds::Bounds3f;

pub trait Shape {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        unimplemented!()
    }
}

pub struct ShapeBase {

}