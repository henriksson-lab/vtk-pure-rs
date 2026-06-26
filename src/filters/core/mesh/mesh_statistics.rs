//! Comprehensive mesh statistics.

use crate::data::PolyData;

/// Comprehensive mesh statistics report.
pub struct MeshStats {
    pub num_points: usize,
    pub num_cells: usize,
    pub num_triangles: usize,
    pub num_quads: usize,
    pub num_other: usize,
    pub num_edges: usize,
    pub num_boundary_edges: usize,
    pub euler_characteristic: isize,
    pub is_closed: bool,
    pub surface_area: f64,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
}

/// Compute comprehensive mesh statistics.
pub fn mesh_statistics(mesh: &PolyData) -> MeshStats {
    let num_points = mesh.points.len();
    let mut num_triangles = 0;
    let mut num_quads = 0;
    let mut num_other = 0;
    let num_cells = mesh.total_cells();

    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut area = 0.0;

    num_other += mesh.verts.num_cells() + mesh.lines.num_cells() + mesh.strips.num_cells();

    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            let a = edge[0] as usize;
            let b = edge[1] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    for cell in mesh.polys.iter() {
        match cell.len() {
            3 => num_triangles += 1,
            4 => num_quads += 1,
            _ => num_other += 1,
        }
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
        // Area
        if cell.len() >= 3 {
            let p0 = mesh.points.get(cell[0] as usize);
            for i in 1..cell.len() - 1 {
                let p1 = mesh.points.get(cell[i] as usize);
                let p2 = mesh.points.get(cell[i + 1] as usize);
                let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                let cx = e1[1] * e2[2] - e1[2] * e2[1];
                let cy = e1[2] * e2[0] - e1[0] * e2[2];
                let cz = e1[0] * e2[1] - e1[1] * e2[0];
                area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
            }
        }
    }

    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for tri in strip.windows(3) {
            for edge in [[tri[0], tri[1]], [tri[1], tri[2]], [tri[2], tri[0]]] {
                let a = edge[0] as usize;
                let b = edge[1] as usize;
                *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
            let p0 = mesh.points.get(tri[0] as usize);
            let p1 = mesh.points.get(tri[1] as usize);
            let p2 = mesh.points.get(tri[2] as usize);
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
    }

    let num_edges = edge_count.len();
    let num_boundary = edge_count.values().filter(|&&c| c == 1).count();
    let v = num_points as isize;
    let e = num_edges as isize;
    let f = num_cells as isize;
    let euler = v - e + f;

    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..num_points {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    if num_points == 0 {
        mn = [0.0; 3];
        mx = [0.0; 3];
    }

    MeshStats {
        num_points,
        num_cells,
        num_triangles,
        num_quads,
        num_other,
        num_edges,
        num_boundary_edges: num_boundary,
        euler_characteristic: euler,
        is_closed: num_boundary == 0,
        surface_area: area,
        bounds_min: mn,
        bounds_max: mx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single_tri() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = mesh_statistics(&mesh);
        assert_eq!(s.num_points, 3);
        assert_eq!(s.num_cells, 1);
        assert_eq!(s.num_triangles, 1);
        assert_eq!(s.num_edges, 3);
        assert_eq!(s.num_boundary_edges, 3);
        assert!(!s.is_closed);
    }
    #[test]
    fn test_two_tris() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let s = mesh_statistics(&mesh);
        assert_eq!(s.num_points, 4);
        assert_eq!(s.num_cells, 2);
        assert_eq!(s.num_edges, 5);
        assert_eq!(s.num_boundary_edges, 4);
        assert_eq!(s.euler_characteristic, 1); // V-E+F = 4-5+2 = 1
    }

    #[test]
    fn includes_polydata_line_and_strip_cells() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[0, 1, 3, 2]);

        let s = mesh_statistics(&mesh);
        assert_eq!(s.num_cells, 2);
        assert_eq!(s.num_triangles, 0);
        assert_eq!(s.num_other, 2);
        assert_eq!(s.num_edges, 5);
        assert!((s.surface_area - 1.0).abs() < 1e-12);
    }
}
