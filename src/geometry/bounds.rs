use nalgebra::{Vector2};
use nalgebra::Point2;
use num::Bounded;
use crate::Scalar;

pub struct Bounds2<S: Scalar> {
    pub min: Point2<S>,
    pub max: Point2<S>
}

impl<S: Scalar> Bounds2<S> {

    pub fn empty() -> Self {
        Self {
            min: Point2::max_value(),
            max: Point2::min_value()
        }
    }

    pub fn with_bounds(min: Point2<S>, max: Point2<S>) -> Self {
        Self { min, max }
    }

    pub fn diagonal(&self) -> Vector2<S> {
        self.max - self.min
    }

    pub fn area(&self) -> S {
        let d = self.diagonal();
        d.x * d.y
    }
}