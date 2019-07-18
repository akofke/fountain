use crate::{Point2f, Float, Ray, Bounds2f, Point2i, Transformable, Point3f, lerp, INFINITY, RayDifferential, Vec2i, Vec2f, Differential};
use crate::geometry::Transform;
use cgmath::InnerSpace;

#[derive(Clone, Copy)]
pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: Float
}

pub trait Camera: Sync {
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

        // TDOD: depth of field

        let time = lerp(sample.time, self.shutter_interval.0, self.shutter_interval.1);
        let ray = Ray { origin, dir, time, t_max: INFINITY };
        let ray = ray.transform(self.camera_to_world);
        (1.0, ray)
    }
}