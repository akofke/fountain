use std::fs::File;
use std::path::Path;

mod math;
mod geometry;

use math::{Vector3f};
use std::io::BufWriter;
use std::io::Write;
use std::f32;
use crate::geometry::Sphere;

fn main() {
    let width = 1024;
    let height = 768;
    let fov = f32::consts::PI / 3.0;
    let mut framebuf: Vec<Vector3f> = Vec::with_capacity(width * height);
    let sphere = Sphere::new(Vector3f::new(-3.0, 0.0, -16.0), 2.0);
    for j in 0..height {
        for i in 0..width {
            let x = (i as f32 + 0.5) - width as f32 / 2.0;
            let y = -(j as f32 + 0.5) + height as f32 / 2.0;
            let z = -(height as f32) / (2.0 * f32::tan(fov as f32 / 2.0));
            let dir = Vector3f::new(x, y, z).normalize();
            framebuf.push(cast_ray(&Vector3f::new(0.0, 0.0, 0.0), &dir, &sphere));
        }
    }

    write_ppm(width, height, &framebuf, "test.ppm").expect("Failed to write file");
}

fn cast_ray(orig: &Vector3f, dir: &Vector3f, sphere: &Sphere) -> Vector3f {
    if let Some(sphere_dist) = sphere.ray_intersect(orig, dir) {
        Vector3f::new(0.4, 0.4, 0.3)
    } else {
        Vector3f::new(0.2, 0.7, 0.8)
    }
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
