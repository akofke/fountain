
use raytracer::{mipmap::ImageWrap, imageio::ImageTexInfo, Float, Point2f};
use raytracer::imageio;
use raytracer::imageio::{spectrum_to_image, load_image};
use std::path::{PathBuf, Path};
use raytracer::spectrum::Spectrum;

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let fname = Path::new(&path).file_stem().unwrap().to_str().unwrap();
    let info = ImageTexInfo::new(path.clone(), ImageWrap::Repeat, 1.0, true, false);
    let mipmap = imageio::get_mipmap(info)?;

    for blocked_img in mipmap.pyramid() {
        let dims = blocked_img.dimensions();
        let img = blocked_img.to_vec();
        let rgb = spectrum_to_image(&img, dims);
        rgb.save(format!("mipmaps/{}{}x{}.png", fname, dims.0, dims.1))?;
    }

    let (w, h) = mipmap.resolution();
    let mut resampled = vec![Spectrum::uniform(0.0); w * h];
    for i in 0..w {
        for j in 0..h {
            let s = i as Float / w as Float;
            let t = j as Float / h as Float;
            let filtered = mipmap.lookup_trilinear_width(Point2f::new(s, t), 0.0);
            resampled[j * w + i] = filtered;
        }
    }
    let rgb = spectrum_to_image(&resampled, (w, h));
    rgb.save("resampled.png")?;

    let (img, dims) = load_image(path)?;
    let rgb = spectrum_to_image(&img, dims);
    rgb.save("roundtrip.png")?;
    Ok(())
}