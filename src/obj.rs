//! A small, hand-written reader for OBJ model files.
//!
//! OBJ is a text format. Each line starts with a keyword. We read two of them:
//!
//! ```text
//! v 1.0 2.0 3.0      a vertex position (x, y, z)
//! f 1 2 3            a face: a list of vertex numbers, counting from 1
//! ```
//!
//! A face corner can also look like `1/5/2` (position/texture/normal) or `1//2`.
//! We only use the first number, the position. We skip every other line
//! (comments `#`, normals `vn`, texture coordinates `vt`, groups, materials, ...).
//!
//! The file's own normals are ignored, and we work out smooth normals ourselves
//! (see `compute_normals`). Many simple models, like our teapot, have none anyway.

use std::path::Path;

use crate::math::Vec3;
use crate::vertex::Vertex;

/// A shape ready to upload to the GPU: corners, and the triangles that join them.
/// `Debug` lets tests print it (`unwrap_err` needs that, to show what it got instead).
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    /// Three numbers per triangle, each one an index into `vertices`.
    /// `u32` instead of `u16`, because models can have more than 65,535 vertices.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// The smallest box that holds every vertex, as (min corner, max corner).
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in &self.vertices {
            let p = Vec3::new(v.position[0], v.position[1], v.position[2]);
            min = min.min(p);
            max = max.max(p);
        }
        (min, max)
    }
}

/// Reads an OBJ file from disk.
///
/// We return `Result<Mesh, String>`: either the mesh, or a message saying what went wrong.
/// The caller decides what to do with an error. Here, `?` means "if this failed,
/// return the error straight away", and `map_err` turns the I/O error into a `String`.
pub fn load(path: &Path) -> Result<Mesh, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Turns the text of an OBJ file into a `Mesh`.
///
/// This is split out from `load` so tests can pass in a string, with no file needed.
pub fn parse(text: &str) -> Result<Mesh, String> {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // `enumerate` gives (0, line), (1, line), ... We add 1 for human line numbers.
    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        // `split_whitespace` handles spaces, tabs and repeated spaces.
        let mut parts = line.split_whitespace();

        // `match` on an `Option<&str>`: `Some("v")` is a vertex line, `None` an empty line.
        match parts.next() {
            Some("v") => {
                let x = parse_float(parts.next(), line_number)?;
                let y = parse_float(parts.next(), line_number)?;
                let z = parse_float(parts.next(), line_number)?;
                positions.push(Vec3::new(x, y, z));
            }
            Some("f") => {
                // Read every corner of the face into a list of 0-based indices.
                let mut corners = Vec::new();
                for corner in parts {
                    corners.push(parse_index(corner, positions.len(), line_number)?);
                }
                if corners.len() < 3 {
                    return Err(format!(
                        "line {line_number}: a face needs at least 3 corners"
                    ));
                }
                // Split the face into triangles that all share the first corner (a "fan").
                // A triangle stays a triangle; a square (0,1,2,3) becomes (0,1,2) and (0,2,3).
                // This works for convex faces, which is what OBJ files nearly always have.
                for k in 1..corners.len() - 1 {
                    indices.extend([corners[0], corners[k], corners[k + 1]]);
                }
            }
            // Anything else (comments, normals, groups, empty lines...) is skipped.
            _ => {}
        }
    }

    if indices.is_empty() {
        return Err("no faces found".to_string());
    }

    let normals = compute_normals(&positions, &indices);
    // `zip` walks two lists side by side, giving pairs (position, normal).
    let vertices = positions
        .iter()
        .zip(&normals)
        .map(|(p, n)| Vertex {
            position: [p.x, p.y, p.z],
            normal: [n.x, n.y, n.z],
        })
        .collect();

    Ok(Mesh { vertices, indices })
}

fn parse_float(part: Option<&str>, line_number: usize) -> Result<f32, String> {
    let text = part.ok_or(format!("line {line_number}: missing number"))?;
    text.parse()
        .map_err(|_| format!("line {line_number}: '{text}' is not a number"))
}

/// Turns one face corner, like `7`, `7/2` or `7/2/3`, into a 0-based vertex index.
///
/// OBJ counts from 1. It also allows negative numbers, which count back from the
/// last vertex read so far: -1 is the newest vertex.
fn parse_index(corner: &str, vertex_count: usize, line_number: usize) -> Result<u32, String> {
    // `split('/')` gives "7", "2", "3". `.next()` takes the first: the position index.
    // `next()` returns an `Option`, so Rust makes us handle `None`, even though
    // `split` always gives at least one piece.
    let text = corner.split('/').next().unwrap_or("");
    let n: i64 = text
        .parse()
        .map_err(|_| format!("line {line_number}: '{corner}' is not a vertex number"))?;

    let count = vertex_count as i64;
    let index = if n > 0 { n - 1 } else { count + n };
    if n == 0 || index < 0 || index >= count {
        return Err(format!(
            "line {line_number}: vertex {n} does not exist (only {vertex_count} so far)"
        ));
    }
    Ok(index as u32)
}

/// Works out a smooth normal for every vertex.
///
/// For each triangle, the cross product of two of its edges points straight out of it.
/// Its length is twice the triangle's area, so big triangles count for more.
/// Each vertex adds up the normals of all the triangles that touch it, and the
/// sum is made length 1. The result changes smoothly across the surface, so the
/// lighting looks smooth instead of showing every flat triangle.
///
/// The direction depends on the order of the corners. With counter-clockwise corners
/// (seen from outside), the normal points outwards, which is what we want.
fn compute_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); positions.len()];

    // `as_chunks::<3>()` views the list as groups of exactly 3 numbers: one triangle each.
    // It returns (groups, leftovers), and there are no leftovers in a valid mesh.
    let (triangles, _) = indices.as_chunks::<3>();
    for triangle in triangles {
        let [a, b, c] = triangle.map(|i| i as usize);
        let face_normal = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
        for i in [a, b, c] {
            normals[i] = normals[i] + face_normal;
        }
    }

    for n in &mut normals {
        // A vertex used by no triangle (or only by zero-area ones) has no direction.
        // Normalizing a zero vector would divide by zero, so give it any valid direction.
        *n = if n.length() > 0.0 {
            n.normalize()
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_triangle() {
        let mesh = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices, vec![0, 1, 2]);
    }

    #[test]
    fn counter_clockwise_triangle_faces_plus_z() {
        // Seen from +z, these corners go round counter-clockwise.
        let mesh = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        for v in &mesh.vertices {
            assert_eq!(v.normal, [0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn splits_a_square_into_two_triangles() {
        let text = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = parse(text).unwrap();
        assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn reads_slashes_negative_indices_and_skips_other_lines() {
        let text = "# a comment\n\
                    v 0 0 0\nv 1 0 0\nv 0 1 0\n\
                    vn 0 0 1\n\
                    vt 0 0\n\
                    g group\n\
                    f 1/1/1 2//1 -1\n";
        let mesh = parse(text).unwrap();
        assert_eq!(mesh.indices, vec![0, 1, 2]);
    }

    #[test]
    fn rejects_a_missing_vertex() {
        let err = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 4\n").unwrap_err();
        assert!(err.contains("line 4"), "{err}");
    }

    #[test]
    fn rejects_a_bad_number() {
        let err = parse("v 0 zero 0\n").unwrap_err();
        assert!(err.contains("line 1"), "{err}");
    }

    #[test]
    fn bounds_hold_every_vertex() {
        let mesh = parse("v -1 0 2\nv 3 -4 0\nv 0 5 1\nf 1 2 3\n").unwrap();
        let (min, max) = mesh.bounds();
        assert_eq!(min, Vec3::new(-1.0, -4.0, 0.0));
        assert_eq!(max, Vec3::new(3.0, 5.0, 2.0));
    }
}
