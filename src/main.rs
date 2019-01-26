use std::fs::File;
use std::path::Path;

mod math;
mod geometry;
mod light;

use math::{Vector3f};
use std::io::BufWriter;
use std::io::Write;
use std::f32;
use crate::geometry::Sphere;
use crate::geometry::HitRecord;
use crate::light::PointLight;

fn main() {
    let width = 1024;
    let height = 768;
    let fov = f32::consts::PI / 3.0;
    let spheres = vec![
        Sphere::new(Vector3f::new(-3.0, 0.0, -16.0), 2.0),
        Sphere::new(Vector3f::new(-1.0, -1.5, -12.0), 2.0),
        Sphere::new(Vector3f::new(1.5, -0.5, -16.0), 3.0),
        Sphere::new(Vector3f::new(7.0, 5.0, -18.0), 4.0),
    ];
    let lights = vec![PointLight {
        position: Vector3f::new(-20.0, 20.0, 20.0),
        intensity: 1.5
    }];

    let framebuf = render(width, height, fov, spheres, lights);

    write_ppm(width, height, &framebuf, "test.ppm").expect("Failed to write file");
}

fn render(width: usize, height: usize, fov: f32, spheres: Vec<Sphere>, lights: Vec<PointLight>) -> Vec<Vector3f> {
    let mut framebuf: Vec<Vector3f> = Vec::with_capacity(width * height);
    for j in 0..height {
        for i in 0..width {
            let x = (i as f32 + 0.5) - width as f32 / 2.0;
            let y = -(j as f32 + 0.5) + height as f32 / 2.0;
            let z = -(height as f32) / (2.0 * f32::tan(fov as f32 / 2.0));
            let dir = Vector3f::new(x, y, z).normalize();
            framebuf.push(cast_ray(&Vector3f::new(0.0, 0.0, 0.0), &dir, &spheres, &lights));
        }
    }
    return framebuf;
}

fn cast_ray(orig: &Vector3f, dir: &Vector3f, spheres: &[Sphere], lights: &[PointLight]) -> Vector3f {
    if let Some(hit_record) = scene_intersect(orig, dir, spheres) {
        let diffuse_color = Vector3f::new(0.4, 0.4, 0.3);
        let mut diffuse_light_intensity = 0.0;
        for light in lights {
            let light_dir = (light.position - hit_record.hit).normalize();
            diffuse_light_intensity += light.intensity * f32::max(0.0, light_dir.dot(&hit_record.normal));
        }
        diffuse_color * diffuse_light_intensity
    } else {
        Vector3f::new(0.2, 0.7, 0.8)
    }
}

fn scene_intersect(orig: &Vector3f, dir: &Vector3f, spheres: &[Sphere]) -> Option<HitRecord> {
    let mut spheres_dist = f32::MAX;
    let mut hit_record = None;
    for sphere in spheres.iter() {
        match sphere.ray_intersect(orig, dir) {
            Some(dist) if dist < spheres_dist => {
                spheres_dist = dist;
                let hit = *orig + (*dir * dist);
                let normal = (hit - sphere.center).normalize();
                hit_record = Some(HitRecord {
                    dist,
                    hit,
                    normal,
                })
            }
            _ => {}
        }
    }
    hit_record
}


fn write_ppm<P: AsRef<Path>>(width: usize, height: usize, framebuffer: &[Vector3f], path: P) -> std::io::Result<()> {
    assert_eq!(framebuffer.len(), width * height);

    let mut f = BufWriter::new(File::create(path)?);

    write!(&mut f, "P6\n{} {}\n255\n", width, height)?;

    for v in framebuffer.iter() {
        f.write_all(&v.to_rgb())?;
    }

    Ok(())
}
