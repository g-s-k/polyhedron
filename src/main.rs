use std::{
    convert::From,
    i8::{MAX, MIN},
};

use image::{GrayImage, ImageBuffer, Luma};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod surface;
mod vec3;

use surface::Surface;

const BACKGROUND: u8 = 0xf;
const AMBIENT_LIGHT: u8 = 0x2f;
const RESOLUTION: u32 = 256;

#[derive(Clone, Copy, Debug)]
enum Randomness {
    Regular = 0,
    Skewed = 1,
    // Messy = 2,
    // Random = 4,
}

#[derive(Debug)]
struct Recipe {
    vertices: u8,
    randomness: Randomness,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            vertices: 4,
            randomness: Randomness::Regular,
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

fn main() {
    let r = Recipe {
        vertices: 8,
        randomness: Randomness::Skewed,
    };
    let s = Surface::from(r);
    println!("surface looks like {:?}", s);

    let count = RESOLUTION.pow(2);
    let pixel_values: Vec<_> = (0..count)
        .into_par_iter()
        .progress_count(count as u64)
        .map(|i| (i % RESOLUTION, i / RESOLUTION))
        .map(|(x, y)| (x, y, get_pixel_value(&s, map_pixel(x, y))))
        .collect();

    let mut img: GrayImage = ImageBuffer::new(RESOLUTION, RESOLUTION);
    for (x, y, value) in pixel_values {
        let pixel = img.get_pixel_mut(x, y);
        *pixel = Luma([value]);
    }

    img.save("tmp.png").expect("failed to save image");
}
