use std::ops::{Add, Div, Mul, Sub};

use crate::types::GCodePlane;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn unit_angle(angle: f32, plane: &GCodePlane) -> Vec3 {
        let axis_1 = angle.cos();
        let axis_2 = angle.sin();
        Vec3::from_2d(axis_1, axis_2, plane)
    }

    pub fn from_2d(axis_1: f32, axis_2: f32, plane: &GCodePlane) -> Vec3 {
        match plane {
            GCodePlane::XY => Vec3 {
                x: axis_1,
                y: axis_2,
                z: 0.,
            },
            GCodePlane::ZX => Vec3 {
                z: axis_1,
                x: axis_2,
                y: 0.,
            },
            GCodePlane::YZ => Vec3 {
                y: axis_1,
                z: axis_2,
                x: 0.,
            },
        }
    }

    pub fn coords_for_plane(&self, plane: &GCodePlane) -> (f32, f32) {
        match plane {
            GCodePlane::XY => (self.x, self.y),
            GCodePlane::ZX => (self.z, self.x),
            GCodePlane::YZ => (self.y, self.z),
        }
    }

    pub fn third_coord(&self, plane: &GCodePlane) -> f32 {
        match plane {
            GCodePlane::XY => self.z,
            GCodePlane::ZX => self.y,
            GCodePlane::YZ => self.x,
        }
    }

    pub fn project_plane(&self, plane: &GCodePlane) -> Vec3 {
        let mut out = *self;
        match plane {
            GCodePlane::XY => out.z = 0.,
            GCodePlane::ZX => out.y = 0.,
            GCodePlane::YZ => out.x = 0.,
        }
        out
    }

    pub fn angle_to(&self, other: &Vec3, plane: &GCodePlane) -> f32 {
        let (a1, a2) = self.coords_for_plane(plane);
        let (b1, b2) = other.coords_for_plane(plane);
        (b2 - a2).atan2(b1 - a1)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        if self.magnitude() == 0. {
            return Vec3::zero();
        }
        *self / self.magnitude()
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
