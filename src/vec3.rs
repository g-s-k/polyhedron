use std::{
    convert::From,
    f64::EPSILON,
    ops::{Add, Div, Mul, Neg, Sub},
};

use super::surface::Point;

fn det2(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * d - b * c
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3(pub f64, pub f64, pub f64);

type Plane<'a> = (&'a Vec3, &'a Vec3, &'a Vec3);

impl From<&Point> for Vec3 {
    fn from(&Point(x, y, z): &Point) -> Self {
        Self(x as f64, y as f64, z as f64)
    }
}

impl<T, U, V> From<(T, U, V)> for Vec3
where
    f64: From<T> + From<U> + From<V>,
{
    fn from((x, y, z): (T, U, V)) -> Self {
        Self(f64::from(x), f64::from(y), f64::from(z))
    }
}

impl Add<Self> for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, divisor: f64) -> Self::Output {
        Self(self.0 / divisor, self.1 / divisor, self.2 / divisor)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, factor: f64) -> Self::Output {
        Self(self.0 * factor, self.1 * factor, self.2 * factor)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3(rhs.0 * self, rhs.1 * self, rhs.2 * self)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}

impl Sub<Self> for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl Vec3 {
    pub fn dot(&self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    fn cross(&self, other: Self) -> Self {
        Self(
            det2(self.1, self.2, other.1, other.2),
            -det2(self.0, self.2, other.0, other.2),
            det2(self.0, self.1, other.0, other.1),
        )
    }

    fn mag(&self) -> f64 {
        self.dot(*self).sqrt()
    }

    pub fn norm(&self) -> Self {
        *self / self.mag()
    }

    fn normal((a, b, c): Plane) -> Self {
        let u = *b - *a;
        let v = *c - *a;

        u.cross(v).norm()
    }

    fn project(&self, onto: Self) -> Self {
        self.dot(onto) / onto.dot(onto) * onto
    }

    pub fn is_in_face(&self, (a, b, c): Plane) -> bool {
        let u = *b - *a;
        let v = *c - *a;
        let p = *self - *a;

        // if the point is coplanar with the face the projection of (self - a)
        // onto the normal vector of ABC should be zero. give it some tolerance
        // because floats.
        let abc_normal = u.cross(v);
        let onto_abc_normal = p.project(abc_normal);

        if onto_abc_normal.mag() > 1. {
            return false;
        }

        // project p onto plane ABC
        let projection_abc = p - onto_abc_normal;

        // see if the projection lies inside the triangle on the plane
        // see http://blackpawn.com/texts/pointinpoly/
        let umg = u.dot(u);
        let vmg = v.dot(v);
        let udv = u.dot(v);
        let udp = u.dot(projection_abc);
        let vdp = v.dot(projection_abc);
        let denom = vmg * umg - udv.powi(2);

        let v_coeff = (umg * vdp - udv * udp) / denom;
        let u_coeff = (vmg * udp - udv * vdp) / denom;

        v_coeff >= -EPSILON && u_coeff >= -EPSILON && u_coeff + v_coeff <= 1.
    }

    pub fn reflect(&self, p: Plane) -> Self {
        let norm = Self::normal(p);
        let proj = self.project(norm);
        let orthogonal_component = *self - proj;
        proj - orthogonal_component
    }
}
