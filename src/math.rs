//! A small, hand-written 3D math module: a `Vec3` and a 4x4 matrix (`Mat4`).
//!
//! We only write what the renderer needs. A full library (like `glam`) has much more,
//! but writing it ourselves shows how the matrices really work.
//!
//! Conventions (these must match what wgpu and the shader expect):
//! - Right-handed: +x is right, +y is up, and the camera looks down -z.
//! - Depth goes from 0 (near) to 1 (far). This is what wgpu, Vulkan and DX12 use.
//!   (OpenGL uses -1 to 1, so OpenGL formulas from tutorials are slightly different.)
//! - Column-major: a matrix is stored as 4 columns. WGSL's `mat4x4<f32>` reads it the same way.

use std::ops::{Add, Mul, Sub};

/// A 3D vector (a point or a direction).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// Dot product: how much two vectors point the same way.
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product: a vector at right angles to both inputs.
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// The same direction, but with length 1.
    pub fn normalize(self) -> Vec3 {
        self * (1.0 / self.length())
    }

    /// The smaller of each part (x, y, z) of the two vectors.
    pub fn min(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    /// The larger of each part (x, y, z) of the two vectors.
    pub fn max(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

// Implementing the `Sub` trait lets us write `a - b` for two `Vec3`s.
// Rust operators are just traits: `-` calls `Sub::sub`, `*` calls `Mul::mul`.
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// `v * 2.0`: makes a vector longer or shorter. The right-hand side is an `f32`,
// so this is `Mul<f32>`, a different trait from `Mul<Mat4>` below.
impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

/// A 4x4 matrix, stored as 4 columns of 4 numbers: `cols[column][row]`.
///
/// `#[repr(C)]` and `Pod` let us copy it straight into a GPU buffer, like `Vertex`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mat4 {
    pub cols: [[f32; 4]; 4],
}

impl Mat4 {
    /// The matrix that changes nothing (like multiplying by 1).
    pub const IDENTITY: Mat4 = Mat4 {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Moves points by `offset`. The offset goes in the last column, which is
    /// multiplied by `w`. Points have w = 1 so they move; directions have w = 0 so they don't.
    pub fn translation(offset: Vec3) -> Mat4 {
        Mat4 {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [offset.x, offset.y, offset.z, 1.0],
            ],
        }
    }

    /// Makes things `s` times bigger, the same amount in every direction.
    pub fn scale(s: f32) -> Mat4 {
        Mat4 {
            cols: [
                [s, 0.0, 0.0, 0.0],
                [0.0, s, 0.0, 0.0],
                [0.0, 0.0, s, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Turns points around the y axis (up) by `angle` radians.
    pub fn rotation_y(angle: f32) -> Mat4 {
        let (s, c) = angle.sin_cos();
        Mat4 {
            cols: [
                [c, 0.0, -s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Turns points around the x axis (right) by `angle` radians.
    // Not used by the renderer right now (the teapot only turns around y), but kept
    // for later. `allow(dead_code)` stops the "never used" warning.
    #[allow(dead_code)]
    pub fn rotation_x(angle: f32) -> Mat4 {
        let (s, c) = angle.sin_cos();
        Mat4 {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, s, 0.0],
                [0.0, -s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// The view (camera) matrix: moves the world so the camera sits at the origin,
    /// looking down -z, with `up` pointing up on screen.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        // Three directions, all at right angles: forward, right and up for the camera.
        let f = (target - eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);
        Mat4 {
            cols: [
                [r.x, u.x, -f.x, 0.0],
                [r.y, u.y, -f.y, 0.0],
                [r.z, u.z, -f.z, 0.0],
                [-r.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }

    /// The projection matrix: adds perspective, so far things look smaller.
    ///
    /// - `fov_y`: how wide the camera sees, top to bottom, in radians.
    /// - `aspect`: window width / height. This stops the picture from stretching.
    /// - `near`, `far`: anything closer than `near` or further than `far` is cut off.
    ///
    /// It puts the distance into `w`. The GPU then divides x, y and z by `w`,
    /// and that division is what makes far things smaller.
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y / 2.0).tan();
        Mat4 {
            cols: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, far / (near - far), -1.0],
                [0.0, 0.0, near * far / (near - far), 0.0],
            ],
        }
    }
}

// `a * b` means "do `b` first, then `a`". So `proj * view * model` moves a point
// by the model matrix, then the view, then the projection.
impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, other: Mat4) -> Mat4 {
        let mut out = [[0.0; 4]; 4];
        for (col, out_col) in out.iter_mut().enumerate() {
            for (row, value) in out_col.iter_mut().enumerate() {
                // Row `row` of `self` times column `col` of `other`.
                *value = (0..4).map(|k| self.cols[k][row] * other.cols[col][k]).sum();
            }
        }
        Mat4 { cols: out }
    }
}

// Tests only compile for `cargo test`. Run them with `cargo test math`.
#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    /// Multiplies the matrix by a point (x, y, z, w).
    fn transform(m: Mat4, p: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0; 4];
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..4).map(|k| m.cols[k][row] * p[k]).sum();
        }
        out
    }

    fn assert_close(a: [f32; 4], b: [f32; 4]) {
        for i in 0..4 {
            assert!((a[i] - b[i]).abs() < 1e-5, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn identity_changes_nothing() {
        let m = Mat4::rotation_y(0.7);
        assert_eq!(Mat4::IDENTITY * m, m);
        assert_eq!(m * Mat4::IDENTITY, m);
    }

    #[test]
    fn translation_moves_points_but_not_directions() {
        let m = Mat4::translation(Vec3::new(1.0, 2.0, 3.0));
        assert_close(transform(m, [1.0, 1.0, 1.0, 1.0]), [2.0, 3.0, 4.0, 1.0]);
        assert_close(transform(m, [1.0, 1.0, 1.0, 0.0]), [1.0, 1.0, 1.0, 0.0]);
    }

    #[test]
    fn scale_then_translate() {
        // `a * b` does `b` first: scale by 2, then move up by 1.
        let m = Mat4::translation(Vec3::new(0.0, 1.0, 0.0)) * Mat4::scale(2.0);
        assert_close(transform(m, [1.0, 1.0, 1.0, 1.0]), [2.0, 3.0, 2.0, 1.0]);
    }

    #[test]
    fn rotation_y_turns_x_towards_minus_z() {
        // A quarter turn to the left (seen from above) moves +x to -z.
        let p = transform(Mat4::rotation_y(FRAC_PI_2), [1.0, 0.0, 0.0, 1.0]);
        assert_close(p, [0.0, 0.0, -1.0, 1.0]);
    }

    #[test]
    fn rotation_x_turns_y_towards_z() {
        let p = transform(Mat4::rotation_x(FRAC_PI_2), [0.0, 1.0, 0.0, 1.0]);
        assert_close(p, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn look_at_moves_target_in_front_of_camera() {
        let view = Mat4::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // The camera itself ends up at the origin.
        assert_close(transform(view, [0.0, 0.0, 5.0, 1.0]), [0.0, 0.0, 0.0, 1.0]);
        // The target is 5 units in front, which is -z.
        assert_close(transform(view, [0.0, 0.0, 0.0, 1.0]), [0.0, 0.0, -5.0, 1.0]);
    }

    #[test]
    fn perspective_maps_near_to_0_and_far_to_1() {
        let proj = Mat4::perspective(1.0, 1.5, 0.1, 100.0);
        let near = transform(proj, [0.0, 0.0, -0.1, 1.0]);
        let far = transform(proj, [0.0, 0.0, -100.0, 1.0]);
        // Depth is z / w after the GPU's divide.
        assert!((near[2] / near[3]).abs() < 1e-5);
        assert!((far[2] / far[3] - 1.0).abs() < 1e-5);
    }
}
