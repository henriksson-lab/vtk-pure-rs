//! Sierpinski triangle and tetrahedron fractals.

use crate::data::{CellArray, Points, PolyData};

/// Generate a 2D Sierpinski triangle with given depth.
pub fn sierpinski_triangle(depth: usize) -> PolyData {
    let v0 = [0.0, 0.0, 0.0];
    let v1 = [1.0, 0.0, 0.0];
    let v2 = [0.5, 3.0f64.sqrt() / 2.0, 0.0];
    let mut tris: Vec<[[f64; 3]; 3]> = vec![[v0, v1, v2]];

    for _ in 0..depth {
        let mut next = Vec::new();
        for [a, b, c] in &tris {
            let ab = mid(*a, *b);
            let bc = mid(*b, *c);
            let ca = mid(*c, *a);
            next.push([*a, ab, ca]);
            next.push([ab, *b, bc]);
            next.push([ca, bc, *c]);
            // Center triangle removed
        }
        tris = next;
    }

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for [a, b, c] in &tris {
        let i = pts.len();
        pts.push(*a);
        pts.push(*b);
        pts.push(*c);
        polys.push_cell(&[i as i64, (i + 1) as i64, (i + 2) as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Generate a 3D Sierpinski tetrahedron with given depth.
pub fn sierpinski_tetrahedron(depth: usize) -> PolyData {
    let v0 = [1.0, 1.0, 1.0];
    let v1 = [1.0, -1.0, -1.0];
    let v2 = [-1.0, 1.0, -1.0];
    let v3 = [-1.0, -1.0, 1.0];
    let mut tets: Vec<[[f64; 3]; 4]> = vec![[v0, v1, v2, v3]];

    for _ in 0..depth {
        let mut next = Vec::new();
        for [a, b, c, d] in &tets {
            let ab = mid(*a, *b);
            let ac = mid(*a, *c);
            let ad = mid(*a, *d);
            let bc = mid(*b, *c);
            let bd = mid(*b, *d);
            let cd = mid(*c, *d);
            next.push([*a, ab, ac, ad]);
            next.push([ab, *b, bc, bd]);
            next.push([ac, bc, *c, cd]);
            next.push([ad, bd, cd, *d]);
        }
        tets = next;
    }

    // Convert tetrahedra to surface triangles
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for [a, b, c, d] in &tets {
        let i = pts.len();
        pts.push(*a);
        pts.push(*b);
        pts.push(*c);
        pts.push(*d);
        let faces = [[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]];
        for f in &faces {
            polys.push_cell(&[(i + f[0]) as i64, (i + f[1]) as i64, (i + f[2]) as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_triangle_0() {
        let s = sierpinski_triangle(0);
        assert_eq!(s.polys.num_cells(), 1);
    }
    #[test]
    fn test_triangle_2() {
        let s = sierpinski_triangle(2);
        assert_eq!(s.polys.num_cells(), 9); // 3^2
    }
    #[test]
    fn test_tetrahedron_1() {
        let s = sierpinski_tetrahedron(1);
        assert_eq!(s.polys.num_cells(), 16); // 4 tets * 4 faces
    }
}
