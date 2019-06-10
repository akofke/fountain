use nalgebra::{Vector2, Point3, Vector3};
use nalgebra::Point2;
use num::Bounded;
use crate::Scalar;
use std::fmt::Error;
use crate::geometry::Ray;

pub type Bounds2f = Bounds2<f32>;
pub type Bounds2i = Bounds2<i32>;
pub type Bounds3f = Bounds3<f32>;

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub struct Bounds3<S: Scalar> {
    pub min: Point3<S>,
    pub max: Point3<S>
}

impl <S: Scalar> Bounds3<S> {
    pub fn with_bounds(min: Point3<S>, max: Point3<S>) -> Self {
        Self {min, max}
    }

    pub fn empty() -> Self {
        Self::with_bounds(Point3::max_value(), Point3::min_value())
    }

    pub fn join(&self, other: &Self) -> Self {
        Self::with_bounds(
            Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            )

        )
    }

    pub fn join_point(&self, point: &Point3<S>) -> Self {
        Self::with_bounds(
            Point3::new(
                self.min.x.min(point.x),
                self.min.y.min(point.y),
                self.min.z.min(point.z),
            ),

            Point3::new(
                self.max.x.max(point.x),
                self.max.y.max(point.y),
                self.max.z.max(point.z),
            )
        )
    }

    pub fn centroid(&self) -> Point3<S> {
        self.min + (self.diagonal() / S::from(2))
    }

    pub fn diagonal(&self) -> Vector3<S> {
        self.max - self.min
    }

    pub fn maximum_extent(&self) -> u8 {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn is_point(&self) -> bool {
        self.max == self.min
    }
}

impl<F: num::Float + Scalar> Bounds3<F> {

    pub fn offset(&self, p: &Point3<F>) -> Vector3<F> {
        let mut o = p - self.min;
        if self.max.x > self.min.x { o.x /= self.max.x - self.min.x };
        if self.max.y > self.min.y { o.y /= self.max.y - self.min.y };
        if self.max.z > self.min.z { o.z /= self.max.z - self.min.z };
        o
    }

    pub fn intersect_test(&self, ray: &Ray) -> Option<(f32, f32)> {
        unimplemented!()
    }

}

impl<S: Scalar> std::fmt::Debug for Bounds3<S>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), Error> {
        let arrmin: [S; 3] = self.min.coords.into();
        let arrmax: [S; 3] = self.max.coords.into();
        write!(f, "Aabb[{:?}, {:?}]", arrmin, arrmax)
    }
}
