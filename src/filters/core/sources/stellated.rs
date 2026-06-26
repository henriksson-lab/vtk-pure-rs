//! Stellated polyhedra (star shapes from regular polyhedra).

use crate::data::{CellArray, Points, PolyData};

/// Create a stellated octahedron (stella octangula).
pub fn stella_octangula(radius: f64) -> PolyData {
    // Two interlocking tetrahedra
    let r = radius;
    let v1 = [[r, r, r], [r, -r, -r], [-r, r, -r], [-r, -r, r]];
    let v2 = [[-r, -r, -r], [-r, r, r], [r, -r, r], [r, r, -r]];
    let faces = [[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for v in &v1 {
        pts.push(*v);
    }
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
    }
    let offset = pts.len() as i64;
    for v in &v2 {
        pts.push(*v);
    }
    for f in &faces {
        polys.push_cell(&[
            f[2] as i64 + offset,
            f[1] as i64 + offset,
            f[0] as i64 + offset,
        ]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a star polyhedron by extending faces outward.
pub fn stellate_mesh(mesh: &PolyData, extension: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..mesh.points.len() {
        pts.push(mesh.points.get(i));
    }

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            polys.push_cell(cell);
            continue;
        }
        // Compute face centroid and normal
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &v in cell {
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let nf = cell.len() as f64;
        cx /= nf;
        cy /= nf;
        cz /= nf;
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let mut n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-15 {
            n[0] /= l;
            n[1] /= l;
            n[2] /= l;
        }
        // Add apex point
        let apex = pts.len() as i64;
        pts.push([
            cx + n[0] * extension,
            cy + n[1] * extension,
            cz + n[2] * extension,
        ]);
        // Create triangles from each edge to apex
        let nc = cell.len();
        for i in 0..nc {
            polys.push_cell(&[cell[i], cell[(i + 1) % nc], apex]);
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
    fn test_stella() {
        let s = stella_octangula(1.0);
        assert_eq!(s.points.len(), 8);
        assert_eq!(s.polys.num_cells(), 8);
    }
    #[test]
    fn test_stellate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = stellate_mesh(&mesh, 0.5);
        assert_eq!(r.polys.num_cells(), 6); // 2 faces * 3 tris each
    }
}
