//! Mesh thickness estimation: ray-based wall thickness measurement.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate wall thickness by casting rays inward from each vertex along its normal.
pub fn estimate_thickness(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let normals = compute_normals(mesh);
    let all_cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|cell| valid_cell(cell, n))
        .map(|c| c.to_vec())
        .collect();

    let mut thickness = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        let dir = [-nm[0], -nm[1], -nm[2]]; // inward
        let mut min_t = f64::MAX;
        for cell in &all_cells {
            if cell.len() < 3 {
                continue;
            }
            if cell.iter().any(|&pid| pid as usize == i) {
                continue;
            } // skip self
            let a = mesh.points.get(cell[0] as usize);
            for triangle_id in 1..cell.len() - 1 {
                let b = mesh.points.get(cell[triangle_id] as usize);
                let c = mesh.points.get(cell[triangle_id + 1] as usize);
                if let Some(t) = ray_tri(p, dir, a, b, c) {
                    if t > 0.01 && t < min_t {
                        min_t = t;
                    }
                }
            }
        }
        thickness.push(if min_t < f64::MAX { min_t } else { 0.0 });
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Thickness",
            thickness,
            1,
        )));
    result
}

/// Compute min/max/mean thickness.
pub fn thickness_stats(mesh: &PolyData) -> (f64, f64, f64) {
    let arr = match mesh.point_data().get_array("Thickness") {
        Some(a) => a,
        None => return (0.0, 0.0, 0.0),
    };
    let mut buf = [0.0f64];
    let mut min_t = f64::MAX;
    let mut max_t = 0.0f64;
    let mut sum = 0.0;
    let mut count = 0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.0 {
            min_t = min_t.min(buf[0]);
            max_t = max_t.max(buf[0]);
            sum += buf[0];
            count += 1;
        }
    }
    if count > 0 {
        (min_t, max_t, sum / count as f64)
    } else {
        (0.0, 0.0, 0.0)
    }
}

fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_cell(cell, n) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let fn_ = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        for &pid in cell {
            let idx = pid as usize;
            for c in 0..3 {
                nm[idx][c] += fn_[c];
            }
        }
    }
    for n in &mut nm {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            for c in 0..3 {
                n[c] /= len;
            }
        }
    }
    nm
}

fn valid_cell(cell: &[i64], number_of_points: usize) -> bool {
    cell.iter()
        .all(|&id| id >= 0 && (id as usize) < number_of_points)
}

fn ray_tri(o: [f64; 3], d: [f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> Option<f64> {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    Some(f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cube_thickness() {
        // Cube should have thickness ≈ 1.0 (opposite face distance)
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
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
        );
        let result = estimate_thickness(&mesh);
        assert!(result.point_data().get_array("Thickness").is_some());
        let (_min, _max, mean) = thickness_stats(&result);
        assert!(mean > 0.5 && mean < 2.0, "mean thickness={mean}");
    }
}
