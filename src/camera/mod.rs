use crate::{Point2f, Float, Ray, Bounds2f, Point2i, Transformable, Point3f, lerp, INFINITY, RayDifferential, Vec2i, Vec2f, Differential};
use crate::geometry::Transform;
use cgmath::InnerSpace;

#[derive(Clone, Copy, Debug)]
pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: Float
}

pub trait Camera: Sync + Send {
    fn generate_ray(&self, sample: CameraSample) -> (Float, Ray);

    fn generate_ray_differential(&self, sample: CameraSample) -> (Float, RayDifferential) {
        let (mut weight, ray) = self.generate_ray(sample);

        let cs_shift_x = CameraSample { p_film: sample.p_film + Vec2f::new(1.0, 0.0), ..sample};
        let (wtx, rx) = self.generate_ray(cs_shift_x);

        let cs_shift_y = CameraSample { p_film: sample.p_film + Vec2f::new(0.0, 1.0), ..sample};
        let (wty, ry) = self.generate_ray(cs_shift_y);

        let ray_diff = RayDifferential {
            ray,
            diff: Some(Differential {
                rx_origin: rx.origin,
                rx_dir: rx.dir,
                ry_origin: ry.origin,
                ry_dir: ry.dir,
            })
        };

        if wtx == 0.0 || wty == 0.0 {
            weight = 0.0
        }
        (weight, ray_diff)
    }
}

struct CameraProjection {
    pub camera_to_screen: Transform,
    pub screen_to_raster: Transform,
    pub raster_to_camera: Transform,
    pub raster_to_screen: Transform,
}

impl CameraProjection {
    fn new(
        camera_to_screen: Transform,
        full_resolution: Point2i,
        screen_window: Bounds2f,
    ) -> Self {
        let screen_to_raster =
            Transform::scale(full_resolution.x as Float, full_resolution.y as Float, 1.0) *
            Transform::scale(
                1.0 / (screen_window.max.x - screen_window.min.x),
                1.0 / (screen_window.min.y - screen_window.max.y),
                1.0
            ) *
            Transform::translate(vec3f!(-screen_window.min.x, -screen_window.max.y, 0.0));

        let raster_to_screen = screen_to_raster.inverse();
        let raster_to_camera = camera_to_screen.inverse() * raster_to_screen;

        Self { camera_to_screen, screen_to_raster, raster_to_camera, raster_to_screen }
    }
}

pub struct PerspectiveCamera {
    camera_to_world: Transform,
    proj: CameraProjection,
    shutter_interval: (Float, Float),
    lens_radius: Float,
    focal_dist: Float,
    aspect: Float
}

impl PerspectiveCamera {
    // TODO: figure out why screen_window has to be [-1, 1]
    pub fn new(
        camera_to_world: Transform,
        full_resolution: Point2i,
        screen_window: Bounds2f,
        shutter_interval: (Float, Float),
        lens_radius: Float,
        focal_dist: Float,
        fov: Float
    ) -> Self {
        let persp = Transform::perspective(fov, 0.001, 1000.0);
        let proj = CameraProjection::new(persp, full_resolution, screen_window);
        let mut p_min: Point3f = point3f!(0, 0, 0).transform(proj.raster_to_camera);
        let mut p_max: Point3f = point3f!(full_resolution.x, full_resolution.y, 0).transform(proj.raster_to_camera);
        p_min /= p_min.z;
        p_max /= p_max.z;
        let aspect = ((p_max.x - p_min.x) * (p_max.y - p_min.y)).abs();

        Self {
            camera_to_world,
            proj,
            shutter_interval,
            lens_radius,
            focal_dist,
            aspect
        }
    }
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, sample: CameraSample) -> (Float, Ray) {
        let p_film = point3f!(sample.p_film.x, sample.p_film.y, 0);
        let p_camera: Point3f = p_film.transform(self.proj.raster_to_camera);

        let origin = Point3f::new(0.0, 0.0, 0.0);
        let dir = (p_camera - origin).normalize();

        // TODO: depth of field

        let time = lerp(sample.time, self.shutter_interval.0, self.shutter_interval.1);
        let ray = Ray { origin, dir, time, t_max: INFINITY };
        let ray = ray.transform(self.camera_to_world);
        (1.0, ray)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounds3f, Bounds2i, NEG_INFINITY, ComponentWiseExt, Vec3f};
    use crate::sampler::random::RandomSampler;
    use crate::sampler::Sampler;
    use cgmath::num_traits::real::Real;
    use cgmath::{Deg, assert_abs_diff_eq};

    #[test]
    fn test_camera_look_at() {
        let camera_tf = Transform::camera_look_at((0.0, 0.0, 0.0).into(), (0.0, 0.0, 1.0).into(), (0.0, 1.0, 0.0).into());
        let res = (16, 16).into();
        let camera = PerspectiveCamera::new(
            camera_tf,
            res,
            Bounds2f::whole_screen(),
            (0.0, 1.0),
            0.0,
            1.0,
            60.0
        );

        let mut sampler = RandomSampler::new_with_seed(32, 1);
        let px_bounds = Bounds2i::with_bounds((0, 0).into(), res);
        for pixel in px_bounds.iter_points() {
            sampler.start_pixel(pixel.into());

            while sampler.start_next_sample() {
                let camera_sample = sampler.get_camera_sample(pixel.into());
                let (_t, ray) = camera.generate_ray(camera_sample);
                assert!(ray.dir.z > 0.0, format!("{:?}", ray));
            }
        }
    }

    #[test]
    fn test_camera_rays() {
        let camera_tf = Transform::camera_look_at((0.0, 0.0, -1.0).into(), (0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        let fov = 90.0 as Float;
        let res = (64, 64).into();
        let camera = PerspectiveCamera::new(
            camera_tf,
            res,
            Bounds2f::whole_screen(),
            (0.0, 1.0),
            0.0,
            1.0,
            fov
        );

        let frustum_h = dbg!(2.0 * 1.0 * Float::tan(fov.to_radians() / 2.0));
        let half_frust = frustum_h / 2.0;
        let visible_box = Bounds3f::with_bounds(
            (-half_frust, -half_frust, 0.0).into(),
            (half_frust, half_frust, 0.01).into()
        );

        let non_filling_box = Bounds3f::with_bounds(
            (-half_frust + 0.1, -half_frust + 0.1, 0.0).into(),
            (half_frust - 0.1, half_frust - 0.1, 0.01).into()
        );

        let behind_box = Bounds3f::with_bounds(
            (-100.0, -100.0, -1.01).into(),
            (100.0, 100.0, -50.0).into()
        );

        let out_of_fov_box = Bounds3f::with_bounds(
            (half_frust + 0.1, half_frust + 0.1, 0.0).into(),
            (100.0, 100.0, 0.1).into()
        );

        let barely_in_view_box = Bounds3f::with_bounds(
            (half_frust - 0.1, half_frust - 1.0, 0.0).into(),
            (100.0, 100.0, 0.1).into()
        );

        let mut sampler = RandomSampler::new_with_seed(32, 1);

        let mut hit_barely_box = false;
        let mut missed_non_filling_box = false;
        let px_bounds = Bounds2i::with_bounds((0, 0).into(), res);
        for pixel in px_bounds.iter_points() {
            sampler.start_pixel(pixel.into());

            while sampler.start_next_sample() {
                let camera_sample = sampler.get_camera_sample(pixel.into());
                let (_t, ray) = camera.generate_ray(camera_sample);

                assert!(visible_box.intersect_test(&ray).is_some(), format!("{:?} {:?}", camera_sample, ray));
                assert!(behind_box.intersect_test(&ray).is_none(), format!("{:?} {:?}", camera_sample, ray));
                assert!(out_of_fov_box.intersect_test(&ray).is_none(), format!("{:?} {:?}", camera_sample, ray));

                if barely_in_view_box.intersect_test(&ray).is_some() {
                    hit_barely_box = true;
                }

                if non_filling_box.intersect_test(&ray).is_none() {
                    missed_non_filling_box = true;
                }
            }
        }

        assert!(missed_non_filling_box);
        assert!(hit_barely_box);
    }

    #[test]
    fn test_camera_covers_fov() {
        let pos = (0.0, 0.0, -1.0).into();
        let camera_tf = Transform::camera_look_at(pos, (0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        let fov = 90.0 as Float;
        let res = (64, 64).into();
        let camera = PerspectiveCamera::new(
            camera_tf,
            res,
            Bounds2f::whole_screen(),
            (0.0, 1.0),
            0.0,
            1.0,
            fov
        );
        let mut sampler = RandomSampler::new_with_seed(32, 1);
        let px_bounds = Bounds2i::with_bounds((0, 0).into(), res);

        let plane = Bounds3f::with_bounds(
            (-100.0, -100.0, 0.0).into(),
            (100.0, 100.0, 0.01).into()
        );

        let mut min = Point3f::new(INFINITY, INFINITY, INFINITY);
        let mut max = Point3f::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
        for pixel in px_bounds.iter_points() {
            sampler.start_pixel(pixel.into());

            while sampler.start_next_sample() {
                let camera_sample = sampler.get_camera_sample(pixel.into());
                let (_t, ray) = camera.generate_ray(camera_sample);
                let (t0, t1) = plane.intersect_test(&ray).unwrap();
                let p = ray.at(t0);
                min = min.min(p);
                max = max.max(p);
            }
        }

        let top = Point3f::new(0.0, max.y, 0.0) - pos;
        let bottom = Point3f::new(0.0, min.y, 0.0) - pos;
        let left = Point3f::new(min.x, 0.0, 0.0) - pos;
        let right = Point3f::new(max.x, 0.0, 0.0) - pos;

        let angle: Deg<_> = Vec3f::angle(top, bottom).into();
        assert_abs_diff_eq!(angle, Deg(fov), epsilon = 0.01);

        let angle: Deg<_> = Vec3f::angle(right, left).into();
        assert_abs_diff_eq!(angle, Deg(fov), epsilon = 0.01);
    }
}