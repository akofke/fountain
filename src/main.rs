use std::fs::File;
use std::path::Path;

mod math;

use math::{Vector3f};
use std::io::BufWriter;
use std::io::Write;

fn main() {
    let width = 1024;
    let height = 768;
    let mut framebuf: Vec<Vector3f> = Vec::with_capacity(width * height);
    for j in 0..height {
        for i in 0..width {
            framebuf.push(Vector3f::new(j as f32 / height as f32, i as f32 / width as f32, 0.0));
        }
    }

    write_ppm(width, height, &framebuf, "test.ppm").expect("Failed to write file");
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
