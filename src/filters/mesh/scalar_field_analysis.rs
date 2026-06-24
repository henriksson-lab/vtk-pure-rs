//! Scalar field analysis on meshes: critical points, gradient lines, level sets.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Find critical points (minima, maxima, saddles) of a scalar field on a mesh.
pub fn find_scalar_critical_points(mesh: &PolyData, array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let adj = build_adj(mesh, n);
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut crit_pts = Points::<f64>::new();
    let mut crit_type = Vec::new(); // 0=min, 1=max, 2=saddle

    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let vi = values[i];
        let n_lower = adj[i].iter().filter(|&&j| values[j] < vi).count();
        let n_higher = adj[i].iter().filter(|&&j| values[j] > vi).count();
        let n_total = adj[i].len();

        if n_lower == n_total {
            // local minimum
            crit_pts.push(mesh.points.get(i));
            crit_type.push(0.0);
        } else if n_higher == n_total {
            // local maximum
            crit_pts.push(mesh.points.get(i));
            crit_type.push(1.0);
        } else if n_lower > 0 && n_higher > 0 {
            // Check for saddle: alternating lower/higher around ring
            let mut changes = 0;
            let ring: Vec<bool> = adj[i].iter().map(|&j| values[j] > vi).collect();
            for k in 0..ring.len() {
                if ring[k] != ring[(k + 1) % ring.len()] {
                    changes += 1;
                }
            }
            if changes >= 4 {
                // saddle has ≥4 sign changes
                crit_pts.push(mesh.points.get(i));
                crit_type.push(2.0);
            }
        }
    }

    let mut result = PolyData::new();
    result.points = crit_pts;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalType",
            crit_type,
            1,
        )));
    result
}

/// Extract level set (iso-contour) of a scalar field on a mesh.
pub fn extract_level_set(mesh: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let mut crossings = Vec::new();
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let va = values[a];
            let vb = values[b];
            if (va - isovalue) * (vb - isovalue) < 0.0 {
                let t = (isovalue - va) / (vb - va);
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                crossings.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
            }
        }
        if crossings.len() >= 2 {
            let i0 = pts.len() as i64;
            pts.push(crossings[0]);
            let i1 = pts.len() as i64;
            pts.push(crossings[1]);
            lines.push_cell(&[i0, i1]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Compute scalar gradient on mesh as per-vertex vectors.
pub fn scalar_gradient_on_mesh(mesh: &PolyData, array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let adj = build_adj(mesh, n);
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut grad = Vec::with_capacity(n * 3);
    for i in 0..n {
        if adj[i].is_empty() {
            grad.extend_from_slice(&[0.0, 0.0, 0.0]);
            continue;
        }
        let pi = mesh.points.get(i);
        let mut gx = 0.0;
        let mut gy = 0.0;
        let mut gz = 0.0;
        let mut w_sum = 0.0;
        for &j in &adj[i] {
            let pj = mesh.points.get(j);
            let dx = pj[0] - pi[0];
            let dy = pj[1] - pi[1];
            let dz = pj[2] - pi[2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d > 1e-15 {
                let dv = values[j] - values[i];
                let w = 1.0 / d;
                gx += w * dv * dx / d;
                gy += w * dv * dy / d;
                gz += w * dv * dz / d;
                w_sum += w;
            }
        }
        if w_sum > 1e-15 {
            grad.extend_from_slice(&[gx / w_sum, gy / w_sum, gz / w_sum]);
        } else {
            grad.extend_from_slice(&[0.0, 0.0, 0.0]);
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Gradient", grad, 3)));
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn critical_points() {
        let mut pts = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        // Scalar: distance from center → one minimum at center
        let vals: Vec<f64> = (0..100)
            .map(|i| {
                let x = (i % 10) as f64 - 4.5;
                let y = (i / 10) as f64 - 4.5;
                x * x + y * y
            })
            .collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vals, 1)));
        let crits = find_scalar_critical_points(&mesh, "f");
        assert!(crits.points.len() > 0);
    }
    #[test]
    fn level_set() {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                (0..25).map(|i| i as f64).collect(),
                1,
            )));
        let ls = extract_level_set(&mesh, "f", 12.5);
        assert!(ls.lines.num_cells() > 0);
    }
    #[test]
    fn gradient() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.0, 1.0],
                1,
            )));
        let result = scalar_gradient_on_mesh(&mesh, "f");
        assert!(result.point_data().get_array("Gradient").is_some());
    }
}
