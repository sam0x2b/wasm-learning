use std::ops::{Add, Sub, Mul, AddAssign};

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

pub fn corner_vertices(t: Vec2, u: Vec2, v: Vec2, width: f32) -> (Vec2, Vec2) {
    let d = normalize(length(t) * v + length(v) * t);
    let d_perp = Vec2::new(-d.y, d.x);

    let tu = u - t;
    let r = width/2.0;

    let shift = r * d * (length(tu) * length(d_perp) / dot(tu, d_perp));
    (u + shift, u - shift)
}

pub fn dot(u: Vec2, v: Vec2) -> f32{
    u.x * v.x + u.y * v.y
}

pub fn normalize(v: Vec2) -> Vec2 {
    let length = length(v);
    v * (1.0 / length)
}

pub fn length(v: Vec2) -> f32 {
    let magnitude = v.x * v.x + v.y * v.y;
    magnitude.sqrt()
}