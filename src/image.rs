use crate::Vec3;
use crate::to_rgb;
use std::fs::File;
use std::path::Path;
use std::io::{Write, BufWriter};

pub fn write_ppm<P: AsRef<Path>>(width: usize, height: usize, framebuffer: &[Vec3], path: P) -> std::io::Result<()> {
    assert_eq!(framebuffer.len(), width * height);

    let mut f = BufWriter::new(File::create(path)?);

    write!(&mut f, "P6\n{} {}\n255\n", width, height)?;

    for v in framebuffer.into_iter() {
        f.write_all(&to_rgb(*v))?;
    }

    Ok(())
}

pub fn write_ppm_ascii<P: AsRef<Path>>(width: usize, height: usize, framebuffer: &[Vec3], path: P) -> std::io::Result<()> {
    assert_eq!(framebuffer.len(), width * height);

    let mut f = BufWriter::new(File::create(path)?);

    write!(&mut f, "P3\n{} {}\n255\n", width, height)?;

    for v in framebuffer.into_iter() {
        let arr = to_rgb(*v);
        write!(&mut f, "{} {} {}\n", arr[0], arr[1], arr[2])?;
    }

    Ok(())
}
