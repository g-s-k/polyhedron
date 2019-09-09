use std::{
    convert::From,
    i8::{MAX, MIN},
};

use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::ParallelProgressIterator;
use random_color::RandomColor;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod surface;
mod vec3;

use surface::Surface;

const BACKGROUND: u8 = 0xf;
const AMBIENT_LIGHT: u8 = 0x2f;
const RESOLUTION: u32 = 512;

#[derive(Clone, Copy, Debug)]
enum Randomness {
    Regular = 0,
    Skewed = 1,
    Messy = 2,
    // Random = 4,
}

#[derive(Debug)]
struct Recipe {
    vertices: u8,
    scatter: Randomness,
    shape: Randomness,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            vertices: 4,
            scatter: Randomness::Regular,
            shape: Randomness::Regular,
        }
    }
}

fn map_value(v: u32) -> f64 {
    ((v as f64) / (RESOLUTION as f64) * (MAX as f64 - MIN as f64)) + (MIN as f64)
}

fn map_pixel(x: u32, y: u32) -> (f64, f64) {
    (map_value(x), -map_value(y))
}

fn get_pixel_value(s: &Surface, (x, y): (f64, f64)) -> u8 {
    (MIN..MAX)
        .rev()
        .find_map(|z| s.query((x, y, z)))
        .unwrap_or(BACKGROUND)
}

fn scale_channel(a: u32, v: u8) -> u8 {
    (v as f64 / 255. * a as f64) as u8
}

fn scale_color([r, g, b]: &[u32; 3], value: u8) -> [u8; 3] {
    [
        scale_channel(*r, value),
        scale_channel(*g, value),
        scale_channel(*b, value),
    ]
}

fn main() {
    let color = RandomColor::new();
    let vertices = rand::random::<u8>() % 6 + 4;
    println!("{{ color: \"{}\", vertices: {} }}", color.to_hex(), vertices);
    println!();

    let s = Surface::from(Recipe {
        vertices,
        scatter: Randomness::Skewed,
        shape: Randomness::Messy,
    });
    println!("{:?}", s.vertices);

    let count = RESOLUTION.pow(2);
    let pixel_values: Vec<_> = (0..count)
        .into_par_iter()
        .progress_count(count as u64)
        .map(|i| (i % RESOLUTION, i / RESOLUTION))
        .map(|(x, y)| (x, y, get_pixel_value(&s, map_pixel(x, y))))
        .collect();

    let color = color.to_rgb_array();
    let mut img: RgbImage = ImageBuffer::new(RESOLUTION, RESOLUTION);
    for (x, y, value) in pixel_values {
        let pixel = img.get_pixel_mut(x, y);
        if value == BACKGROUND {
            *pixel = Rgb([BACKGROUND, BACKGROUND, BACKGROUND]);
        } else {
            *pixel = Rgb(scale_color(&color, value));
        }
    }

    img.save("tmp.png").expect("failed to save image");
}
