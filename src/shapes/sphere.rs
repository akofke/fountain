use crate::{Float, distance, Normal3, Point3f, Vec3f, Point2f};
use crate::EFloat;
use crate::err_float::gamma;
use crate::interaction::DiffGeom;
use crate::math::quadratic;
use crate::geometry::{Transform, Ray, Transformable};
use crate::shapes::Shape;
use crate::geometry::bounds::Bounds3;
use crate::interaction::SurfaceInteraction;
use cgmath::{InnerSpace};

pub struct Sphere<'t> {
    object_to_world: &'t Transform,
    world_to_object: &'t Transform,
    reverse_orientation: bool,

    radius: Float,
    z_min: Float,
    z_max: Float,
    theta_min: Float,
    theta_max: Float,
    phi_max: Float
}

impl<'t> Sphere<'t> {
    pub fn new(
        object_to_world: &'t Transform,
        world_to_object: &'t Transform,
        reverse_orientation: bool,
        radius: Float,
        z_min: Float,
        z_max: Float,
        phi_max: Float
    ) -> Self {
        Self {
            object_to_world, world_to_object, reverse_orientation,
            radius,
            z_min: Float::min(z_min, z_max).clamp(-radius, radius),

            z_max: Float::max(z_min, z_max).clamp(-radius, radius),
            theta_min: Float::clamp(z_min / radius, -1.0, 1.0).acos(),
            theta_max: Float::clamp(z_max / radius, -1.0, 1.0).acos(),
            phi_max: phi_max.clamp(0.0, 360.0).to_radians()
        }
    }
}

impl<'t> Shape for Sphere<'t> {
    fn object_bound(&self) -> Bounds3<f32> {
        bounds3f!((-self.radius, -self.radius, self.z_min), (self.radius, self.radius, self.z_max))
    }

    fn object_to_world<T: Transformable<O>, O>(&self, t: T) -> O {
        t.transform(*self.object_to_world)
    }

    fn world_to_object<T: Transformable<O>, O>(&self, t: T) -> O {
        t.transform(*self.world_to_object)
    }

    #[allow(non_snake_case)]
    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)> {
        let (ray, origin_err, dir_err) = self.world_to_object(ray);

        let ox = EFloat::with_err(ray.origin.x, origin_err.x);
        let oy = EFloat::with_err(ray.origin.y, origin_err.y);
        let oz = EFloat::with_err(ray.origin.z, origin_err.z);
        let dirx = EFloat::with_err(ray.dir.x, dir_err.x);
        let diry = EFloat::with_err(ray.dir.y, dir_err.y);
        let dirz = EFloat::with_err(ray.dir.z, dir_err.z);

        let a = dirx * dirx + diry * diry + dirz * dirz;
        let b = 2.0 * (dirx * ox + diry * oy + dirz * oz);
        let c = ox * ox + oy * oy + oz * oz - EFloat::new(self.radius) * EFloat::new(self.radius);

        let (t0, t1) = quadratic(a, b, c)?;

        if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
            return None;
        }

        // find the closest valid intersection t value
        let mut t_shape_hit = t0;
        if t_shape_hit.lower_bound() <= 0.0 {
            t_shape_hit = t1;
            if t_shape_hit.upper_bound() > ray.t_max {
                return None
            }
        }

        let mut p_hit = ray.at(t_shape_hit.into());

        p_hit *= self.radius / distance(p_hit, point3f!(0, 0, 0));
        if p_hit.x == 0.0 && p_hit.y == 0.0 { p_hit.x = 1.0e-5 * self.radius }
        let mut phi = Float::atan2(p_hit.y, p_hit.x);
        if phi < 0.0 { phi += 2.0 * std::f32::consts::PI }


        // test against clipping parameters
        if (self.z_min > -self.radius && p_hit.z < self.z_min)
            || (self.z_max < self.radius && p_hit.z > self.z_max)
            || phi > self.phi_max
        {
            if t_shape_hit == t1 { return None; }
            if t1.upper_bound() > ray.t_max { return None; }

            t_shape_hit = t1;

            p_hit = ray.at(t_shape_hit.into());

            p_hit *= self.radius / distance(p_hit, point3f!(0, 0, 0));
            if p_hit.x == 0.0 && p_hit.y == 0.0 { p_hit.x = 1.0e-5 * self.radius }
            phi = Float::atan2(p_hit.y, p_hit.x);
            if phi < 0.0 { phi += 2.0 * std::f32::consts::PI }

            // If we still miss due to clipping
            if (self.z_min > -self.radius && p_hit.z < self.z_min)
                || (self.z_max < self.radius && p_hit.z > self.z_max)
                || phi > self.phi_max
            {
                return None;
            }
        }

        let u = phi / self.phi_max;
        let theta = Float::acos((p_hit.z / self.radius).clamp(-1.0, 1.0));
        let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);

        let z_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        let inv_z_radius = 1.0 / z_radius;
        let cos_phi = p_hit.x * inv_z_radius;
        let sin_phi = p_hit.y * inv_z_radius;

        let dpdu = vec3f!(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = (self.theta_max - self.theta_min) *
            vec3f!(p_hit.z * cos_phi, p_hit.z * sin_phi, -self.radius * theta.sin());

        let d2pduu = (-self.phi_max * self.phi_max) * vec3f!(p_hit.x, p_hit.y, 0.0);
        let d2pduv = (self.theta_max - self.theta_min) * p_hit.z * self.phi_max * vec3f!(-sin_phi, cos_phi, 0.0);
        let d2pdvv = -(self.theta_max - self.theta_min) * (self.theta_max - self.theta_min) *
            vec3f!(p_hit.x, p_hit.y, p_hit.z);

        let E = dpdu.dot(dpdu);
        let F = dpdu.dot(dpdv);
        let G = dpdv.dot(dpdv);

        let N = dpdu.cross(dpdv).normalize();

        let e = N.dot(d2pduu);
        let f = N.dot(d2pduv);
        let g = N.dot(d2pdvv);

        let invEGF2 = 1.0 / (E * G - F * F);

        let dndu = Normal3((f * F - e * G) * invEGF2 * dpdu + (e * F - f * E) * invEGF2 * dpdv);

        let dndv = Normal3((g * F - f * G) * invEGF2 * dpdu + (f * F - g * E) * invEGF2 * dpdv);

        let p_err: Vec3f = gamma(5) * (p_hit - point3f!(0, 0, 0));

        let interact = SurfaceInteraction::new(
            p_hit,
            p_err,
            ray.time,
            Point2f::new(u, v),
            -ray.dir,
            Normal3(N),
            DiffGeom { dpdu, dpdv, dndu, dndv }
        );

        let world_intersect = self.object_to_world(interact);

        Some((t_shape_hit.into(), world_intersect))
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        unimplemented!()
    }
}