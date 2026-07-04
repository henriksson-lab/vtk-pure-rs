//! Normal estimation for unstructured point clouds (no connectivity).

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

/// Estimate normals for a point cloud using PCA of k-nearest neighbors.
pub fn estimate_point_cloud_normals(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let sample_size = k.max(1).min(n);

    let mut normals = Vec::with_capacity(n * 3);
    for i in 0..n {
        let neighbors = knn(&pts, i, sample_size);
        let nm = pca_normal(&pts, &neighbors);
        normals.extend_from_slice(&nm);
    }

    let mut result = mesh.clone();
    replace_normals_array(&mut result, normals);
    result
}

/// Orient point cloud normals toward a reference point.
pub fn orient_point_cloud_normals(mesh: &PolyData, reference: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let normals = match mesh.point_data().normals() {
        Some(nm) if nm.num_components() == 3 && nm.num_tuples() >= n => nm,
        None => return mesh.clone(),
        _ => return mesh.clone(),
    };
    let mut nm_data = Vec::with_capacity(n * 3);
    let mut buf = [0.0f64; 3];

    for i in 0..n {
        normals.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        let to_ref = [
            reference[0] - p[0],
            reference[1] - p[1],
            reference[2] - p[2],
        ];
        let dot = buf[0] * to_ref[0] + buf[1] * to_ref[1] + buf[2] * to_ref[2];
        if dot >= 0.0 {
            nm_data.extend_from_slice(&buf);
        } else {
            nm_data.push(-buf[0]);
            nm_data.push(-buf[1]);
            nm_data.push(-buf[2]);
        }
    }

    let mut result = mesh.clone();
    replace_normals_array(&mut result, nm_data);
    result
}

fn replace_normals_array(mesh: &mut PolyData, normals: Vec<f64>) {
    let source_attrs = mesh.point_data();
    let mut attrs = DataSetAttributes::new();
    let mut replaced = false;
    for array in source_attrs.iter() {
        let name = array.name().to_string();
        if array.name() == "Normals" {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                normals.clone(),
                3,
            )));
            replaced = true;
        } else {
            attrs.add_array(array.clone());
        }
        copy_active_attribute(source_attrs, &mut attrs, &name);
    }
    if !replaced {
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    }
    attrs.set_active_normals("Normals");
    *mesh.point_data_mut() = attrs;
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
}

fn knn(pts: &[[f64; 3]], query: usize, k: usize) -> Vec<usize> {
    let q = pts[query];
    let mut dists: Vec<(usize, f64)> = pts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                i,
                (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2),
            )
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dists.iter().take(k).map(|(i, _)| *i).collect()
}

fn pca_normal(pts: &[[f64; 3]], neighbors: &[usize]) -> [f64; 3] {
    if neighbors.is_empty() {
        return [0.0, 0.0, 1.0];
    }

    let mut mean = [0.0; 3];
    for &ni in neighbors {
        mean[0] += pts[ni][0];
        mean[1] += pts[ni][1];
        mean[2] += pts[ni][2];
    }
    let inv_n = 1.0 / neighbors.len() as f64;
    mean[0] *= inv_n;
    mean[1] *= inv_n;
    mean[2] *= inv_n;

    let mut cov = [[0.0; 3]; 3];
    for &ni in neighbors {
        let d = [
            pts[ni][0] - mean[0],
            pts[ni][1] - mean[1],
            pts[ni][2] - mean[2],
        ];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c] * inv_n;
            }
        }
    }
    smallest_eigenvector(cov)
}

fn smallest_eigenvector(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max_val = a[0][1].abs();
        for r in 0..3 {
            for c in r + 1..3 {
                let val = a[r][c].abs();
                if val > max_val {
                    max_val = val;
                    p = r;
                    q = c;
                }
            }
        }
        if max_val < 1e-15 {
            break;
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let theta = if (app - aqq).abs() < 1e-20 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };
        let cs = theta.cos();
        let sn = theta.sin();

        let mut next = a;
        next[p][p] = cs * cs * app + 2.0 * cs * sn * apq + sn * sn * aqq;
        next[q][q] = sn * sn * app - 2.0 * cs * sn * apq + cs * cs * aqq;
        next[p][q] = 0.0;
        next[q][p] = 0.0;
        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                next[r][p] = cs * arp + sn * arq;
                next[p][r] = next[r][p];
                next[r][q] = -sn * arp + cs * arq;
                next[q][r] = next[r][q];
            }
        }
        a = next;

        for row in &mut v {
            let vp = row[p];
            let vq = row[q];
            row[p] = cs * vp + sn * vq;
            row[q] = -sn * vp + cs * vq;
        }
    }

    let mut min_idx = 0usize;
    for i in 1..3 {
        if a[i][i] <= a[min_idx][min_idx] {
            min_idx = i;
        }
    }

    let n = [v[0][min_idx], v[1][min_idx], v[2][min_idx]];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-20 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;
    #[test]
    fn planar_normals() {
        let pts: Vec<[f64; 3]> = (0..20)
            .map(|i| [(i % 5) as f64, (i / 5) as f64, 0.0])
            .collect();
        let mut mesh = PolyData::new();
        mesh.points = Points::from(pts);
        let result = estimate_point_cloud_normals(&mesh, 5);
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(10, &mut buf);
        // Normal of XY plane should be approximately [0,0,±1]
        assert!(buf[2].abs() > 0.8, "normal z={}", buf[2]);
    }
    #[test]
    fn orient() {
        let pts: Vec<[f64; 3]> = (0..10)
            .map(|i| [(i % 5) as f64, (i / 5) as f64, 0.0])
            .collect();
        let mut mesh = PolyData::from_points(pts);
        mesh = estimate_point_cloud_normals(&mesh, 3);
        let result = orient_point_cloud_normals(&mesh, [2.0, 1.0, 1.0]);
        assert!(result.point_data().normals().is_some());
    }
}
