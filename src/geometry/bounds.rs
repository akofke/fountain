use cgmath::{Point2, Vector2, Point3, Vector3};
use num::Bounded;
use crate::{Scalar, Vec3f};
use std::fmt::Error;
use crate::geometry::Ray;
use std::mem::swap;
use crate::err_float::gamma;

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

    pub fn intersection(&self, other: &Bounds2<S>) -> Bounds2<S> {
        let min = Point2::<S>::new(
            S::max(self.min.x, other.min.x),
            S::max(self.min.y, other.min.y),
        );
        let max = Point2::<S>::new(
            S::min(self.max.x, other.max.x),
            S::min(self.max.y, other.max.y),
        );
        Self::with_bounds(min, max)
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
        self.min + (self.diagonal() / std::convert::From::from(2))
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

impl Bounds3<f32> {

    pub fn offset(&self, p: &Point3<f32>) -> Vec3f {
        let mut o = p - self.min;
        if self.max.x > self.min.x { o.x /= self.max.x - self.min.x };
        if self.max.y > self.min.y { o.y /= self.max.y - self.min.y };
        if self.max.z > self.min.z { o.z /= self.max.z - self.min.z };
        o
    }

    pub fn intersect_test(&self, ray: &Ray) -> Option<(f32, f32)> {
        let mut t0 = 0.0f32;
        let mut t1 = ray.t_max;

        for i in 0..3 {
            let inv_ray_dir = 1.0 / ray.dir[i];
            let mut t_near = (self.min[i] - ray.origin[i]) * inv_ray_dir;
            let mut t_far = (self.max[i] - ray.origin[i]) * inv_ray_dir;

            if t_near > t_far { swap(&mut t_near, &mut t_far) }

            // expand t_far to account for fp error
            t_far *= 1.0 + 2.0 * gamma(3);

            t0 = f32::max(t0, t_near);
            t1 = f32::min(t1, t_far);
            if t0 > t1 { return None; }
        }
        Some((t0, t1))
    }

    pub fn intersect_test_fast(&self, ray: &Ray) -> Option<(f32, f32)> {
        unimplemented!();
    }

}

impl<S: Scalar> std::fmt::Debug for Bounds3<S>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), Error> {
        let arrmin: [S; 3] = self.min.into();
        let arrmax: [S; 3] = self.max.into();
        write!(f, "Bounds3f[{:?}, {:?}]", arrmin, arrmax)
    }
}


#[cfg(test)]
mod test {
    use crate::geometry::bounds::Bounds3f;
    use crate::geometry::Ray;

    #[test]
    fn test_bounds3f_intersect() {
        // basic hit
        let bounds = bounds3f!((1, 1, 1), (2, 2, 2));
        let ray = Ray::new(point3f!(0, 0, 0), vec3f!(1, 1, 1));

        assert_eq!(bounds.intersect_test(&ray), Some((1.0, 2.0)));

        // zero component
        let bounds = bounds3f!((-0.5, -0.5, -0.5), (0.5, 0.5, 0.5));
        let ray = Ray::new(point3f!(0, 0, -2), vec3f!(0, 0, 1));

        assert_eq!(bounds.intersect_test(&ray), Some((1.5, 2.5)));

        // miss
        let bounds = bounds3f!((1, 1, 1), (2, 2, 2));
        let ray = Ray::new(point3f!(0, 0, 0), vec3f!(-1, 1, 1));

        assert_eq!(bounds.intersect_test(&ray), None);


        // along edge
        let bounds = bounds3f!((1, 1, 1), (2, 2, 2));
        let ray = Ray::new(point3f!(1, 1, 1), vec3f!(1, 0, 0));

        assert_eq!(bounds.intersect_test(&ray), Some((0.0, 1.0)));
    }
}
