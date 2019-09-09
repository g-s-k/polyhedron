use std::{
    convert::From,
    f64::consts::PI,
    i8::{MAX, MIN},
    ops::Add,
};

use super::{vec3::Vec3, Randomness, Recipe, AMBIENT_LIGHT};

const RADIUS: f64 = 96.0;

fn golden_ratio() -> f64 {
    (1. + (5. as f64).sqrt()) / 2.
}

fn clipped_add(a: i8, b: i8) -> i8 {
    a.checked_add(b).unwrap_or(MAX)
}

fn make_random_value(r: Randomness) -> i8 {
    (rand::random::<i8>() % (MAX / 4))
        .checked_mul(r as i8)
        .unwrap_or(MAX)
}

fn randomize_radius(s: Randomness) -> f64 {
    RADIUS
        + (rand::random::<i8>() % (MAX / 8))
            .checked_mul(s as i8)
            .unwrap_or(MAX) as f64
}

#[derive(Debug)]
pub struct Point(pub i8, pub i8, pub i8);

impl Add<Self> for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(
            clipped_add(self.0, other.0),
            clipped_add(self.1, other.1),
            clipped_add(self.2, other.2),
        )
    }
}

type Face<'a> = (&'a Point, &'a Point, &'a Point);

impl Point {
    /// create a point from an even spherical distribution
    ///
    /// see Lattice 3 at http://extremelearning.com.au/evenly-distributing-points-on-a-sphere/
    /// formula is like so:
    ///     (x, y) = ( (i + 6) / (n + 11), i / <golden ratio> )
    ///     (theta, phi) = ( acos(2x - 1) - <pi> / 2, 2<pi>y )

    fn from_index(i: u8, n: u8, shape: Randomness) -> Self {
        let (x, y);
        if i == 0 {
            x = 0.;
            y = 0.;
        } else if i == n - 1 {
            x = 1.;
            y = 0.;
        } else {
            x = (i as f64 + 6.) / (n as f64 + 11.);
            y = i as f64 / golden_ratio();
        }

        let theta = (2. * x - 1.).acos() - PI / 2.;
        let (ts, tc) = theta.sin_cos();

        let phi = 2. * PI * y;
        let (ps, pc) = phi.sin_cos();

        let rho = randomize_radius(shape);
        let (x, y, z) = (rho * tc * pc, rho * tc * ps, rho * ts);

        Self(x as i8, y as i8, z as i8)
    }

    fn random(reg: Randomness) -> Self {
        Self(
            make_random_value(reg),
            make_random_value(reg),
            make_random_value(reg),
        )
    }

    fn in_face((a, b, c): Face, p: Vec3) -> bool {
        p.is_in_face((&a.into(), &b.into(), &c.into()))
    }

    fn direct_light((a, b, c): Face) -> u8 {
        f64::min(
            f64::max(
                (Vec3(0., 0., -1.)
                    .reflect((&a.into(), &b.into(), &c.into()))
                    .norm()
                    .dot(Vec3(-1., -1., 1.).norm())
                    * (0xff - AMBIENT_LIGHT) as f64)
                    .floor(),
                0.,
            ),
            255.,
        ) as u8
    }
}

#[derive(Debug)]
pub struct Surface {
    pub vertices: Vec<Point>,
}

impl From<Recipe> for Surface {
    fn from(r: Recipe) -> Self {
        Self {
            vertices: (0..r.vertices)
                .map(|i| Point::from_index(i, r.vertices, r.shape) + Point::random(r.scatter))
                .collect(),
        }
    }
}

impl Surface {
    fn bbox(&self) -> (Point, Point) {
        let (mut max, mut min) = (Point(MIN, MIN, MIN), Point(MAX, MAX, MAX));

        for &Point(x, y, z) in &self.vertices {
            if x > max.0 {
                max.0 = x;
            }

            if x < min.0 {
                min.0 = x;
            }

            if y > max.1 {
                max.1 = y;
            }

            if y < min.1 {
                min.1 = y;
            }

            if z > max.2 {
                max.2 = z;
            }

            if z < min.2 {
                min.2 = z;
            }
        }

        (max, min)
    }

    pub fn query<T: Into<Vec3>>(&self, pt: T) -> Option<u8> {
        let pt = pt.into();

        let (max, min) = self.bbox();

        if pt.0 > max.0 as f64
            || pt.0 < min.0 as f64
            || pt.1 > max.1 as f64
            || pt.1 < min.1 as f64
            || pt.2 > max.2 as f64
            || pt.2 < min.2 as f64
        {
            return None;
        }

        let mut all = &self.vertices[..];
        while let Some((a, mut rest0)) = all.split_first() {
            all = rest0;
            while let Some((b, mut rest1)) = rest0.split_first() {
                rest0 = rest1;

                while let Some((c, rest2)) = rest1.split_first() {
                    rest1 = rest2;

                    if Point::in_face((a, b, c), pt) {
                        return Some(
                            Point::direct_light((a, b, c))
                                .checked_add(AMBIENT_LIGHT)
                                .unwrap_or(0xff),
                        );
                    }
                }
            }
        }

        None
    }
}
