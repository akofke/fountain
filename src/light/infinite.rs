use std::sync::Arc;
use crate::mipmap::{MIPMap, ImageWrap};
use crate::spectrum::Spectrum;
use crate::sampling::Distribution2D;
use crate::{Point3f, Float, Point2f, RayDifferential, Transform, Vec3f, spherical_phi, spherical_theta, Normal3};
use crate::light::{Light, LiSample, LightFlags, VisibilityTester};
use crate::primitive::Primitive;
use crate::bvh::BVH;
use crate::interaction::SurfaceHit;
use crate::consts;
use cgmath::{EuclideanSpace, InnerSpace};

pub struct InfiniteAreaLight {
    l_map: Arc<MIPMap<Spectrum>>,
    distribution: Distribution2D,

    world_center: Point3f,
    world_radius: Float,
    light_to_world: Transform,
    world_to_light: Transform,
}

impl InfiniteAreaLight {
    pub fn new_envmap(
        envmap: Arc<MIPMap<Spectrum>>,
        light_to_world: Transform,
    ) -> Self {
        let distribution = Self::compute_distribution(&envmap);
        let world_to_light = light_to_world.inverse();

        Self {
            l_map: envmap,
            distribution,

            world_center: Point3f::origin(),
            world_radius: 0.0,
            light_to_world,
            world_to_light,
        }
    }

    pub fn new_uniform(
        radiance: Spectrum,
        light_to_world: Transform,
    ) -> Self {
        let texels = vec![radiance];
        let res = (1, 1);
        let mipmap = MIPMap::new(res, texels, ImageWrap::Repeat);
        let distribution = Self::compute_distribution(&mipmap);
        let world_to_light = light_to_world.inverse();

        Self {
            l_map: Arc::new(mipmap),
            distribution,

            world_center: Point3f::origin(),
            world_radius: 0.0,
            light_to_world,
            world_to_light,
        }
    }

    fn compute_distribution(mipmap: &MIPMap<Spectrum>) -> Distribution2D {
        let (height, width) = mipmap.resolution();
        let filter = 1.0 / (width.max(height) as Float);
        let mut img = vec![0.0; width * height];
        for j in 0..height {
            let v = j as Float / height as Float;
            let sin_theta = (std::f32::consts::PI * (j as Float + 0.5) / height as Float).sin();
            for i in 0..width {
                let u = i as Float / width as Float;
                let luminance = mipmap.lookup_trilinear_width(Point2f::new(u, v), filter).luminance();
                img[i + j * width] = luminance * sin_theta;
            }
        }
        Distribution2D::new(&img, width, height)
    }
}

impl Light for InfiniteAreaLight {
    fn flags(&self) -> LightFlags {
        LightFlags::Infinite
    }

    fn light_to_world(&self) -> &Transform {
        &self.light_to_world
    }

    fn world_to_light(&self) -> &Transform {
        &self.world_to_light
    }

    fn preprocess(&mut self, scene_prims: &BVH<Box<dyn Primitive>>) {
        let (center, radius) = scene_prims.bounds.bounding_sphere();
        self.world_center = center;
        self.world_radius = radius;
    }

    fn sample_incident_radiance(&self, reference: &SurfaceHit, u: Point2f) -> LiSample {
        let (uv, map_pdf) = self.distribution.sample_continuous(u);
        if map_pdf == 0.0 {
            unimplemented!()
        }

        // map (u, v) sample to spherical coordinates
        let theta = uv.y * consts::PI;
        let phi = uv.x * 2.0 * consts::PI;
        // convert sample point to direction
        let wi = self.light_to_world.transform(Vec3f::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos()
        ));

        let pdf = if theta.sin() == 0.0 {
            0.0
        } else {
            map_pdf / (2.0 * consts::PI * consts::PI * theta.sin())
        };

        let vis = VisibilityTester {
            p0: *reference,
            p1: SurfaceHit {
                p: reference.p + wi * (2.0 * self.world_radius),
                p_err: Vec3f::new(0.0, 0.0, 0.0),
                time: reference.time,
                n: Normal3::zero(),
            },
        };

        // TODO: illuminant, width?
        let radiance = self.l_map.lookup_trilinear_width(uv, 0.0);

        LiSample {
            radiance,
            wi,
            pdf,
            vis
        }
    }

    fn pdf_incident_radiance(&self, _reference: &SurfaceHit, wi: Vec3f) -> Float {
        let wi = self.world_to_light.transform(wi);
        let theta = spherical_theta(wi);
        let phi = spherical_phi(wi);
        if theta.sin() == 0.0 {
            0.0
        } else {
            self.distribution.pdf(Point2f::new(
                phi * (1.0 / (2.0 * consts::PI)),
                theta * consts::FRAC_1_PI
            )) / (2.0 * consts::PI * consts::PI * theta.sin())
        }
    }

    fn environment_emitted_radiance(&self, ray: &RayDifferential) -> Spectrum {
        let w = self.world_to_light.transform(ray.ray.dir).normalize();
        let st = Point2f::new(
            spherical_phi(w) * (1.0 / (2.0 * std::f32::consts::PI)),
            spherical_theta(w) * std::f32::consts::FRAC_1_PI
        );
        // TODO: Illuminant SpectrumType for full spectral mode
        self.l_map.lookup_trilinear_width(st, 0.0)
    }
}