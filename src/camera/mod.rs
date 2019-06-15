use crate::{Point2f, Float, Ray, Transform, Bounds2f, Point2i};

#[derive(Clone, Copy)]
pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: Float
}

pub trait Camera {
    fn generate_ray(&self, sample: CameraSample) -> (Float, Ray);
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