use std::fs::File;
use std::path::Path;

mod geometry;
mod light;

use std::io::BufWriter;
use std::io::Write;
use std::f32;
use crate::{
    light::PointLight,
    geometry::{Sphere, Vec3, HitRecord, Object, Ray}
};
use nalgebra::clamp;
use num::cast::ToPrimitive;

pub fn to_rgb(v: Vec3) -> [u8; 3] {
    let mut arr = [0u8; 3];
    let bytes = v.map(|x| {
        let clamped = clamp(x, 0.0, 1.0) * 255.0;
        clamped.to_u8().unwrap()
    });
    arr.copy_from_slice(bytes.as_slice());
    arr
}

fn background(dir: &Vec3) -> Vec3 {
    // scale so t is between 0.0 and 1.0
    let t = 0.5 * (dir[1] + 1.0);
    // linear interpolation based on t
    (1.0 - t) * Vec3::repeat(1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn main() {
    let width = 1024;
    let height = 768;
    let fov = f32::consts::PI / 3.0;
    let spheres = vec![
        Sphere::new(Vec3::new(-3.0, 0.0, -16.0), 2.0),
        Sphere::new(Vec3::new(-1.0, -1.5, -12.0), 2.0),
        Sphere::new(Vec3::new(1.5, -0.5, -16.0), 3.0),
        Sphere::new(Vec3::new(7.0, 5.0, -18.0), 4.0),
        Sphere::new(Vec3::new(0.0, 0.0, -3.0), 0.5),
        Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0) // horizon-ish
    ];
    let lights = vec![PointLight {
        position: Vec3::new(-20.0, 20.0, 20.0),
        intensity: 1.5
    }];

    let framebuf = render(width, height, fov, spheres, lights);

    write_ppm_ascii(width, height, &framebuf, "test2.ppm").expect("Failed to write file");
}

fn render(width: usize, height: usize, fov: f32, scene: impl Object, lights: Vec<PointLight>) -> Vec<Vec3> {
    let mut framebuf: Vec<Vec3> = Vec::with_capacity(width * height);
    for j in 0..height {
        for i in 0..width {
            let x = (i as f32 + 0.5) - width as f32 / 2.0;
            let y = -(j as f32 + 0.5) + height as f32 / 2.0;
            let z = -(height as f32) / (2.0 * f32::tan(fov as f32 / 2.0));
            let dir = Vec3::new(x, y, z).normalize();
            let ray = Ray {origin: Vec3::zeros(), dir};
            framebuf.push(cast_ray(&ray, &scene, &lights));
        }
    }
    return framebuf;
}

fn cast_ray(ray: &Ray, scene: &impl Object, lights: &[PointLight]) -> Vec3 {
    if let Some(hit_record) = scene.hit(ray, 0.0, f32::MAX) {
        return (hit_record.normal + Vec3::repeat(1.0)) * 0.5; // normal map


//        let diffuse_color = Vec3::new(0.4, 0.4, 0.3);
//        let mut diffuse_light_intensity = 0.0;
//        for light in lights {
//            let light_dir = (light.position - hit_record.hit).normalize();
//            diffuse_light_intensity += light.intensity * f32::max(0.0, light_dir.dot(&hit_record.normal));
//        }
//        diffuse_color * diffuse_light_intensity
    } else {
        background(&ray.dir)
    }
}

fn write_ppm<P: AsRef<Path>>(width: usize, height: usize, framebuffer: &[Vec3], path: P) -> std::io::Result<()> {
    assert_eq!(framebuffer.len(), width * height);

    let mut f = BufWriter::new(File::create(path)?);

    write!(&mut f, "P6\n{} {}\n255\n", width, height)?;

    for v in framebuffer.into_iter() {
        f.write_all(&to_rgb(*v))?;
    }

    Ok(())
}

fn write_ppm_ascii<P: AsRef<Path>>(width: usize, height: usize, framebuffer: &[Vec3], path: P) -> std::io::Result<()> {
    assert_eq!(framebuffer.len(), width * height);

    let mut f = BufWriter::new(File::create(path)?);

    write!(&mut f, "P3\n{} {}\n255\n", width, height)?;

    for v in framebuffer.into_iter() {
        let arr = to_rgb(*v);
        write!(&mut f, "{} {} {}\n", arr[0], arr[1], arr[2])?;
    }

    Ok(())
}
