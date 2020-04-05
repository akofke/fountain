use std::path::Path;
use crate::spectrum::Spectrum;
use crate::Float;
use std::fs::File;
use std::io::{Write, Seek, BufReader};
use exr::prelude::*;
use exr::image::simple::{Image, Samples, Channel, Layer};
use std::convert::TryInto;
use smallvec::smallvec;

pub fn read_exr(path: impl AsRef<Path>) -> anyhow::Result<(Vec<Spectrum>, (usize, usize))> {
    let file = BufReader::new(File::open(path)?);
    let image = Image::read_from_buffered(file, read_options::default()).unwrap();

    let layer = &image.layers[0];
    let Vec2(w, h) = layer.data_size;
    let r = Text::from("R").unwrap();
    let b = Text::from("B").unwrap();
    let g = Text::from("G").unwrap();
    let r_chan = layer.channels.iter().find(|c| c.name == r).unwrap();
    let b_chan = layer.channels.iter().find(|c| c.name == b).unwrap();
    let g_chan = layer.channels.iter().find(|c| c.name == g).unwrap();

    let pixels: Vec<Spectrum> = match (&r_chan.samples, &g_chan.samples, &b_chan.samples) {
        (Samples::F16(r), Samples::F16(g), Samples::F16(b)) => {
            r.iter().zip(g.iter()).zip(b.iter())
                .map(|((r, g), b)| {
                    Spectrum::from([r.to_f32(), g.to_f32(), b.to_f32()])
                })
                .collect()
        },
        (Samples::F32(r), Samples::F32(g), Samples::F32(b)) => {
            r.iter().zip(g.iter()).zip(b.iter())
                .map(|((r, g), b)| {
                    Spectrum::from([*r, *g, *b])
                })
                .collect()
        }
        _ => {
            panic!()
        }
    };

    Ok((pixels, (w as usize, h as usize)))
}

pub fn write_exr<W: Write + Seek>(writer: &mut W, img: Vec<Spectrum>, dims: (u32, u32)) -> anyhow::Result<()> {
    let (w, h) = dims;
    let size = w * h;
    let mut r_samples = Vec::with_capacity(size as usize);
    let mut g_samples = Vec::with_capacity(size as usize);
    let mut b_samples = Vec::with_capacity(size as usize);

    img.into_iter()
        .for_each(|p| {
            let [r, g, b] = p.into_array();
            r_samples.push(r);
            g_samples.push(g);
            b_samples.push(b);
        });

    let r_chan = Channel::new_linear(
        "R".try_into().unwrap(),
        Samples::F32(r_samples),
    );
    let g_chan = Channel::new_linear(
        "G".try_into().unwrap(),
        Samples::F32(g_samples),
    );
    let b_chan = Channel::new_linear(
        "B".try_into().unwrap(),
        Samples::F32(b_samples),
    );

    let layer = Layer::new(
        "image".try_into().unwrap(),
        Vec2(w as usize, h as usize),
        smallvec![r_chan, g_chan, b_chan]
    );
    let layer = layer
        .with_compression(Compression::RLE)
        .with_block_format(None, LineOrder::Increasing);

    let image = Image::new_from_single_layer(layer);
    image.write_to_buffered(writer, write_options::default()).unwrap();
    Ok(())
}