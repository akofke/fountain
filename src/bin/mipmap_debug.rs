
use raytracer::{mipmap::ImageWrap, imageio::ImageTexInfo};
use raytracer::imageio;
use raytracer::imageio::{spectrum_to_image, load_image};
use std::path::{PathBuf, Path};

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let fname = Path::new(&path).file_stem().unwrap().to_str().unwrap();
    let info = ImageTexInfo::new(path.clone(), ImageWrap::Repeat, 1.0, true);
    let mipmap = imageio::get_mipmap(info)?;

    for blocked_img in mipmap.pyramid() {
        let dims = blocked_img.dimensions();
        let img = blocked_img.to_vec();
        let rgb = spectrum_to_image(&img, dims);
        rgb.save(format!("mipmaps/{}{}x{}.png", fname, dims.0, dims.1))?;
    }

    let (img, dims) = load_image(path)?;
    let rgb = spectrum_to_image(&img, dims);
    rgb.save("roundtrip.png")?;
    Ok(())
}