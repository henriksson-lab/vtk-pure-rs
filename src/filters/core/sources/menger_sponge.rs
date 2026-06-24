//! Menger sponge fractal geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a Menger sponge fractal at given depth.
pub fn menger_sponge(depth: usize) -> PolyData {
    let mut cubes: Vec<([f64; 3], f64)> = vec![([0.0, 0.0, 0.0], 1.0)];

    for _ in 0..depth {
        let mut next = Vec::new();
        for &(origin, size) in &cubes {
            let s = size / 3.0;
            for ix in 0..3 {
                for iy in 0..3 {
                    for iz in 0..3 {
                        let holes = [ix == 1, iy == 1, iz == 1].iter().filter(|&&h| h).count();
                        if holes >= 2 {
                            continue;
                        } // remove center cross
                        next.push((
                            [
                                origin[0] + ix as f64 * s,
                                origin[1] + iy as f64 * s,
                                origin[2] + iz as f64 * s,
                            ],
                            s,
                        ));
                    }
                }
            }
        }
        cubes = next;
    }

    // Build mesh from cubes
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for &(o, s) in &cubes {
        let base = pts.len();
        let verts = [
            [o[0], o[1], o[2]],
            [o[0] + s, o[1], o[2]],
            [o[0] + s, o[1] + s, o[2]],
            [o[0], o[1] + s, o[2]],
            [o[0], o[1], o[2] + s],
            [o[0] + s, o[1], o[2] + s],
            [o[0] + s, o[1] + s, o[2] + s],
            [o[0], o[1] + s, o[2] + s],
        ];
        for v in &verts {
            pts.push(*v);
        }
        let faces = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ];
        for f in &faces {
            polys.push_cell(&[
                (base + f[0]) as i64,
                (base + f[1]) as i64,
                (base + f[2]) as i64,
                (base + f[3]) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_depth_0() {
        let m = menger_sponge(0);
        assert_eq!(m.points.len(), 8); // single cube
        assert_eq!(m.polys.num_cells(), 6);
    }
    #[test]
    fn test_depth_1() {
        let m = menger_sponge(1);
        assert_eq!(m.points.len(), 20 * 8); // 20 sub-cubes
        assert_eq!(m.polys.num_cells(), 20 * 6);
    }
    #[test]
    fn test_depth_2() {
        let m = menger_sponge(2);
        assert_eq!(m.points.len(), 400 * 8); // 20^2 sub-cubes
    }
}
