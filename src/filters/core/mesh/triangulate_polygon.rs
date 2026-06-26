//! Triangulate arbitrary polygons using ear clipping.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use crate::types::Scalar;

/// Triangulate all polygons in a mesh using fan triangulation.
/// Works correctly for convex polygons; approximate for concave ones.
pub fn triangulate_polygons(mesh: &PolyData) -> PolyData {
    let mut new_polys = CellArray::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        let old_cell_id = poly_cell_offset + poly_id;
        if cell.len() <= 3 {
            new_polys.push_cell(cell);
            old_poly_ids.push(old_cell_id);
        } else {
            // Fan triangulation from first vertex
            let v0 = cell[0];
            for i in 1..cell.len() - 1 {
                new_polys.push_cell(&[v0, cell[i], cell[i + 1]]);
                old_poly_ids.push(old_cell_id);
            }
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    remap_cell_data(mesh, &old_poly_ids, &mut result);
    result
}

/// Triangulate using ear clipping, following VTK's polygon triangulation path.
pub fn triangulate_ear_clip(mesh: &PolyData) -> PolyData {
    let mut new_polys = CellArray::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        let old_cell_id = poly_cell_offset + poly_id;
        if cell.len() <= 3 {
            new_polys.push_cell(cell);
            old_poly_ids.push(old_cell_id);
            continue;
        }
        let pts: Vec<[f64; 3]> = cell.iter().map(|&v| mesh.points.get(v as usize)).collect();
        let indices: Vec<i64> = cell.to_vec();
        let tris = ear_clip_2d(&pts, &indices);
        for tri in tris {
            new_polys.push_cell(&tri);
            old_poly_ids.push(old_cell_id);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    remap_cell_data(mesh, &old_poly_ids, &mut result);
    result
}

fn ear_clip_2d(pts: &[[f64; 3]], indices: &[i64]) -> Vec<[i64; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[indices[0], indices[1], indices[2]]];
    }
    if n == 4 {
        if simple_polygon_quad_uses_d1(pts) {
            return vec![
                [indices[0], indices[1], indices[2]],
                [indices[0], indices[2], indices[3]],
            ];
        }
        return vec![
            [indices[0], indices[1], indices[3]],
            [indices[1], indices[2], indices[3]],
        ];
    }

    // Project to 2D using dominant normal axis
    let normal = polygon_normal(pts);
    let axis = if normal[0].abs() > normal[1].abs() && normal[0].abs() > normal[2].abs() {
        0
    } else if normal[1].abs() > normal[2].abs() {
        1
    } else {
        2
    };
    let (u_axis, v_axis) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let pts2d: Vec<[f64; 2]> = pts.iter().map(|p| [p[u_axis], p[v_axis]]).collect();
    let orientation = polygon_area_2d(&pts2d).signum();
    if orientation == 0.0 {
        return fallback_triangulation(pts, indices);
    }

    let mut remaining: Vec<usize> = (0..n).collect();
    let mut result = Vec::new();

    let mut max_iter = n * n;
    while remaining.len() > 3 && max_iter > 0 {
        max_iter -= 1;
        let m = remaining.len();
        let mut found = false;
        for i in 0..m {
            let prev = remaining[(i + m - 1) % m];
            let curr = remaining[i];
            let next = remaining[(i + 1) % m];
            if is_ear(&pts2d, &remaining, prev, curr, next, orientation) {
                result.push([indices[prev], indices[curr], indices[next]]);
                remaining.remove(i);
                found = true;
                break;
            }
        }
        if !found {
            return fallback_triangulation(pts, indices);
        }
    }
    if remaining.len() == 3 {
        result.push([
            indices[remaining[0]],
            indices[remaining[1]],
            indices[remaining[2]],
        ]);
    }
    result
}

fn fallback_triangulation(pts: &[[f64; 3]], indices: &[i64]) -> Vec<[i64; 3]> {
    if indices.len() == 4 {
        quad_triangulation(pts, indices)
    } else {
        fan_triangulation(indices)
    }
}

fn quad_triangulation(pts: &[[f64; 3]], indices: &[i64]) -> Vec<[i64; 3]> {
    let diag_02 = dist_sq(pts[0], pts[2]);
    let diag_13 = dist_sq(pts[1], pts[3]);
    if diag_02 <= diag_13 {
        vec![
            [indices[0], indices[1], indices[2]],
            [indices[0], indices[2], indices[3]],
        ]
    } else {
        vec![
            [indices[0], indices[1], indices[3]],
            [indices[1], indices[2], indices[3]],
        ]
    }
}

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn simple_polygon_quad_uses_d1(pts: &[[f64; 3]]) -> bool {
    let normal = polygon_normal(pts);
    let normal_len_sq = dot(normal, normal);
    if normal_len_sq == 0.0 {
        return dist_sq(pts[0], pts[2]) <= dist_sq(pts[1], pts[3]);
    }

    let n012 = cross(sub(pts[1], pts[0]), sub(pts[2], pts[0]));
    let n023 = cross(sub(pts[2], pts[0]), sub(pts[3], pts[0]));
    let d1_ok = dot(n012, normal) > 0.0 && dot(n023, normal) > 0.0;

    let n013 = cross(sub(pts[1], pts[0]), sub(pts[3], pts[0]));
    let n123 = cross(sub(pts[2], pts[1]), sub(pts[3], pts[1]));
    let d2_ok = dot(n013, normal) > 0.0 && dot(n123, normal) > 0.0;

    match (d1_ok, d2_ok) {
        (true, false) => true,
        (false, true) => false,
        _ => dist_sq(pts[0], pts[2]) < dist_sq(pts[1], pts[3]),
    }
}

fn fan_triangulation(indices: &[i64]) -> Vec<[i64; 3]> {
    let mut result = Vec::new();
    if indices.len() < 3 {
        return result;
    }
    let v0 = indices[0];
    for i in 1..indices.len() - 1 {
        result.push([v0, indices[i], indices[i + 1]]);
    }
    result
}

fn polygon_area_2d(pts: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        area += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    area * 0.5
}

fn is_ear(
    pts: &[[f64; 2]],
    remaining: &[usize],
    prev: usize,
    curr: usize,
    next: usize,
    orientation: f64,
) -> bool {
    let a = pts[prev];
    let b = pts[curr];
    let c = pts[next];
    let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if cross * orientation <= 0.0 {
        return false;
    }
    for &idx in remaining {
        if idx == prev || idx == curr || idx == next {
            continue;
        }
        if point_in_triangle(pts[idx], a, b, c) {
            return false;
        }
    }
    true
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn sign(p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]) -> f64 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

fn polygon_normal(pts: &[[f64; 3]]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        n[0] += (pts[i][1] - pts[j][1]) * (pts[i][2] + pts[j][2]);
        n[1] += (pts[i][2] - pts[j][2]) * (pts[i][0] + pts[j][0]);
        n[2] += (pts[i][0] - pts[j][0]) * (pts[i][1] + pts[j][1]);
    }
    n
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn remap_cell_data(input: &PolyData, old_poly_ids: &[usize], output: &mut PolyData) {
    if input.cell_data().num_arrays() == 0 {
        return;
    }

    let mut old_cell_ids = Vec::with_capacity(output.total_cells());
    old_cell_ids.extend(0..input.verts.num_cells());

    let line_offset = input.verts.num_cells();
    old_cell_ids.extend(line_offset..line_offset + input.lines.num_cells());

    old_cell_ids.extend_from_slice(old_poly_ids);

    let strip_offset = input.verts.num_cells() + input.lines.num_cells() + input.polys.num_cells();
    old_cell_ids.extend(strip_offset..strip_offset + input.strips.num_cells());

    output.cell_data_mut().clear();
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(remap_array(array, &old_cell_ids));
        }
    }
}

fn remap_array(array: &AnyDataArray, old_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($array, old_cell_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => remap!(array, F32),
        AnyDataArray::F64(array) => remap!(array, F64),
        AnyDataArray::I8(array) => remap!(array, I8),
        AnyDataArray::I16(array) => remap!(array, I16),
        AnyDataArray::I32(array) => remap!(array, I32),
        AnyDataArray::I64(array) => remap!(array, I64),
        AnyDataArray::U8(array) => remap!(array, U8),
        AnyDataArray::U16(array) => remap!(array, U16),
        AnyDataArray::U32(array) => remap!(array, U32),
        AnyDataArray::U64(array) => remap!(array, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_cell_ids: &[usize]) -> DataArray<T> {
    let mut data = Vec::with_capacity(old_cell_ids.len() * array.num_components());
    for &old_cell_id in old_cell_ids {
        data.extend_from_slice(array.tuple(old_cell_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fan() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_polygons(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_ear_clip() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_ear_clip(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_ear_clip_clockwise_polygon() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_ear_clip(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_polygon_quad_uses_vtk_polygon_path() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_ear_clip(&mesh);
        let cells: Vec<Vec<i64>> = r.polys.iter().map(|cell| cell.to_vec()).collect();
        assert_eq!(cells, vec![vec![0, 1, 3], vec![1, 2, 3]]);
    }
    #[test]
    fn test_triangle_passthrough() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = triangulate_polygons(&mesh);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_triangulation_preserves_lines() {
        let mut mesh = PolyData::from_polyline(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        mesh.polys.push_cell(&[0, 1, 1]);
        let r = triangulate_ear_clip(&mesh);
        assert_eq!(r.lines.num_cells(), 1);
    }
}
