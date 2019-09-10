use std::{
    convert::From,
    i8::{MAX, MIN},
    ops::{Add, Mul},
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

#[derive(Clone, Copy, Debug)]
struct RGB {
    r: u8,
    b: u8,
    g: u8,
}

impl Default for RGB {
    fn default() -> Self {
        Self {
            r: BACKGROUND,
            g: BACKGROUND,
            b: BACKGROUND,
        }
    }
}

impl From<&RandomColor> for RGB {
    fn from(color: &RandomColor) -> Self {
        let [r, g, b] = color.to_rgb_array();

        Self {
            r: r as u8,
            b: b as u8,
            g: g as u8,
        }
    }
}

impl From<RGB> for Rgb<u8> {
    fn from(RGB { r, g, b }: RGB) -> Self {
        Self([r, g, b])
    }
}

impl Add<Self> for RGB {
    type Output = Self;

    fn add(self, Self { r, g, b }: Self) -> Self::Output {
        Self {
            r: self.r.checked_add(r).unwrap_or(255),
            g: self.g.checked_add(g).unwrap_or(255),
            b: self.b.checked_add(b).unwrap_or(255),
        }
    }
}

impl Mul<u8> for RGB {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        let Self { r, g, b } = self;
        Self {
            r: (r as u16 * rhs as u16 / 255 as u16) as u8,
            g: (g as u16 * rhs as u16 / 255 as u16) as u8,
            b: (b as u16 * rhs as u16 / 255 as u16) as u8,
        }
    }
}

impl RGB {
    fn to_hex(&self) -> String {
        format!("#{:x}{:x}{:x}", self.r, self.g, self.b)
    }
}

struct Renderer {
    fg: RGB,
    bg: RGB,
    width: u32,
    height: u32,
}

impl Renderer {
    fn new() -> Self {
        let color_generator = RandomColor::new();
        Self {
            fg: RGB::from(&color_generator),
            bg: RGB::from(&color_generator),
            width: 675,
            height: 675,
        }
    }

    fn get_coord(&self, value: u32, horizontal: bool) -> f64 {
        let n = if horizontal { self.width } else { self.height };

        ((value as f64) / (n as f64) * (MAX as f64 - MIN as f64)) + (MIN as f64)
    }

    fn render(&self, pixels: &[Option<u8>]) -> RgbImage {
        let mut img: RgbImage = ImageBuffer::from_pixel(self.width, self.height, RGB::default().into());

        for (value, pixel) in pixels.iter().zip(img.pixels_mut()) {
            if let Some(v) = value {
                *pixel = Rgb::from(self.fg * *v + self.bg * AMBIENT_LIGHT);
            }
        }

        img
    }
}

fn main() {
    let renderer = Renderer::new();
    let vertices = rand::random::<u8>() % 6 + 5;
    println!(
        "{{ spot: \"{}\", ambient: \"{}\", vertices: {} }}",
        renderer.fg.to_hex(),
        renderer.bg.to_hex(),
        vertices
    );
    println!();

    let s = Surface::from(Recipe {
        vertices,
        scatter: Randomness::Skewed,
        shape: Randomness::Messy,
    });
    println!("{:?}", s.vertices);

    let count = renderer.width * renderer.height;
    let pixel_values: Vec<_> = (0..count)
        .into_par_iter()
        .progress_count(count as u64)
        .map(|i| {
            let (x, y) = (
                renderer.get_coord(i % renderer.width, true),
                -renderer.get_coord(i / renderer.width, false),
            );
            (MIN..MAX).rev().find_map(|z| s.query((x, y, z)))
        })
        .collect();

    renderer
        .render(&pixel_values)
        .save("tmp.png")
        .expect("failed to save image");
}
