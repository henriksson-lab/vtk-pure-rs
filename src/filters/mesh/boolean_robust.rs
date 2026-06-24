//! Robust mesh boolean operations using winding number classification.
//!
//! Provides union, intersection, and difference operations that handle
//! coplanar faces and near-degenerate configurations better than the
//! basic boolean module.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Classify each vertex of mesh A as inside/outside mesh B using
/// generalized winding number.
pub fn classify_vertices(mesh_a: &PolyData, mesh_b: &PolyData) -> Vec<bool> {
    let na = mesh_a.points.len();
    let tris_b = collect_triangles(mesh_b);

    (0..na)
        .map(|i| {
            let p = mesh_a.points.get(i);
            let wn = generalized_winding_number(p, &tris_b);
            wn.abs() > 0.5
        })
        .collect()
}

/// Boolean union: keep faces from A outside B + faces from B outside A.
pub fn boolean_union(a: &PolyData, b: &PolyData) -> PolyData {
    let inside_b = classify_vertices(a, b);
    let inside_a = classify_vertices(b, a);

    let mut result_a = extract_cells_by_vertex_flag(a, &inside_b, false); // outside B
    let result_b = extract_cells_by_vertex_flag(b, &inside_a, false); // outside A

    append_mesh(&mut result_a, &result_b);
    result_a
}

/// Boolean intersection: keep faces from A inside B + faces from B inside A.
pub fn boolean_intersection(a: &PolyData, b: &PolyData) -> PolyData {
    let inside_b = classify_vertices(a, b);
    let inside_a = classify_vertices(b, a);

    let mut result_a = extract_cells_by_vertex_flag(a, &inside_b, true); // inside B
    let result_b = extract_cells_by_vertex_flag(b, &inside_a, true); // inside A

    append_mesh(&mut result_a, &result_b);
    result_a
}

/// Boolean difference: A minus B. Keep faces from A outside B.
pub fn boolean_difference(a: &PolyData, b: &PolyData) -> PolyData {
    let inside_b = classify_vertices(a, b);
    extract_cells_by_vertex_flag(a, &inside_b, false)
}

fn extract_cells_by_vertex_flag(mesh: &PolyData, flags: &[bool], keep_flagged: bool) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        let majority = cell
            .iter()
            .filter(|&&pid| {
                let idx = pid as usize;
                idx < flags.len() && flags[idx]
            })
            .count()
            > cell.len() / 2;

        if majority != keep_flagged {
            continue;
        }

        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(new_idx as i64);
        }
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn append_mesh(target: &mut PolyData, source: &PolyData) {
    let offset = target.points.len() as i64;
    for i in 0..source.points.len() {
        target.points.push(source.points.get(i));
    }
    for cell in source.polys.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
        target.polys.push_cell(&shifted);
    }
}

type Triangle = [[f64; 3]; 3];

fn collect_triangles(mesh: &PolyData) -> Vec<Triangle> {
    let mut tris = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() >= 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            tris.push([a, b, c]);
        }
    }
    tris
}

fn generalized_winding_number(point: [f64; 3], triangles: &[Triangle]) -> f64 {
    let mut wn = 0.0;
    for tri in triangles {
        wn += solid_angle(point, tri);
    }
    wn / (4.0 * std::f64::consts::PI)
}

fn solid_angle(p: [f64; 3], tri: &Triangle) -> f64 {
    let a = [tri[0][0] - p[0], tri[0][1] - p[1], tri[0][2] - p[2]];
    let b = [tri[1][0] - p[0], tri[1][1] - p[1], tri[1][2] - p[2]];
    let c = [tri[2][0] - p[0], tri[2][1] - p[1], tri[2][2] - p[2]];

    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let lc = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();

    if la < 1e-15 || lb < 1e-15 || lc < 1e-15 {
        return 0.0;
    }

    let det = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
        + a[2] * (b[0] * c[1] - b[1] * c[0]);
    let ab = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let ac = a[0] * c[0] + a[1] * c[1] + a[2] * c[2];
    let bc = b[0] * c[0] + b[1] * c[1] + b[2] * c[2];
    let denom = la * lb * lc + ab * lc + ac * lb + bc * la;

    if denom.abs() < 1e-15 {
        return 0.0;
    }
    2.0 * det.atan2(denom)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cube(cx: f64, cy: f64, cz: f64, size: f64) -> PolyData {
        let s = size / 2.0;
        PolyData::from_triangles(
            vec![
                [cx - s, cy - s, cz - s],
                [cx + s, cy - s, cz - s],
                [cx + s, cy + s, cz - s],
                [cx - s, cy + s, cz - s],
                [cx - s, cy - s, cz + s],
                [cx + s, cy - s, cz + s],
                [cx + s, cy + s, cz + s],
                [cx - s, cy + s, cz + s],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        )
    }

    #[test]
    fn classify_inside() {
        let cube = make_cube(0.0, 0.0, 0.0, 2.0);
        let probe = PolyData::from_points(vec![[0.0, 0.0, 0.0], [5.0, 5.0, 5.0]]);
        let flags = classify_vertices(&probe, &cube);
        assert!(flags[0], "center should be inside");
        assert!(!flags[1], "far point should be outside");
    }

    #[test]
    fn union_two_cubes() {
        let a = make_cube(0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1.0, 0.0, 0.0, 2.0);
        let result = boolean_union(&a, &b);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn difference() {
        let a = make_cube(0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1.0, 0.0, 0.0, 2.0);
        let result = boolean_difference(&a, &b);
        assert!(result.polys.num_cells() > 0);
        // Should keep some faces from A that are outside B
        assert!(result.polys.num_cells() <= a.polys.num_cells());
    }

    #[test]
    fn no_overlap() {
        let a = make_cube(0.0, 0.0, 0.0, 1.0);
        let b = make_cube(10.0, 0.0, 0.0, 1.0);
        let inter = boolean_intersection(&a, &b);
        assert_eq!(inter.polys.num_cells(), 0);
    }
}
