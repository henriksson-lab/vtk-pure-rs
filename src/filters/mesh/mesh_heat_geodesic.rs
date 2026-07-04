//! Geodesic distance via the heat method (Crane et al. 2013 simplified).
use std::collections::HashSet;

use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn heat_geodesic(mesh: &PolyData, source: usize, diffusion_steps: usize) -> PolyData {
    let n = mesh.points.len();
    let mut result = mesh.clone();
    if n == 0 || source >= n {
        return result;
    }

    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let Some(a) = valid_point_id(cell[j], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(j + 1) % len], n) else {
                continue;
            };
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    let mut avg_edge = 0.0f64;
    let mut edge_count = 0.0f64;
    for (i, nbrs) in neighbors.iter().enumerate() {
        let pi = mesh.points.get(i);
        for &nb in nbrs {
            if nb > i {
                let pj = mesh.points.get(nb);
                let d =
                    ((pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2))
                        .sqrt();
                avg_edge += d;
                edge_count += 1.0;
            }
        }
    }
    if edge_count < 1.0 {
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "HeatGeodesic",
                vec![0.0; n],
                1,
            )));
        result.point_data_mut().set_active_scalars("HeatGeodesic");
        return result;
    }
    avg_edge /= edge_count;

    let dt = avg_edge * avg_edge;
    let num_diffusion_steps = diffusion_steps.max(1);

    let mut heat = vec![0.0f64; n];
    heat[source] = 1.0;

    for _ in 0..num_diffusion_steps {
        let mut new_heat = vec![0.0f64; n];
        for (i, nbrs) in neighbors.iter().enumerate() {
            if nbrs.is_empty() {
                new_heat[i] = heat[i];
                continue;
            }
            let count = nbrs.len() as f64;
            let mut avg = 0.0f64;
            for &nb in nbrs {
                avg += heat[nb];
            }
            avg /= count;
            new_heat[i] = heat[i] + dt * (avg - heat[i]);
        }
        heat = new_heat;
    }

    let num_faces = mesh.polys.num_cells();
    let mut face_grad: Vec<[f64; 3]> = Vec::with_capacity(num_faces);
    let faces: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    for cell in &faces {
        if cell.len() < 3 {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        }
        let Some([i0, i1, i2]) = valid_triangle_ids(cell, n) else {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        };

        let p0 = mesh.points.get(i0);
        let p1 = mesh.points.get(i1);
        let p2 = mesh.points.get(i2);

        let e01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let normal = [
            e01[1] * e02[2] - e01[2] * e02[1],
            e01[2] * e02[0] - e01[0] * e02[2],
            e01[0] * e02[1] - e01[1] * e02[0],
        ];
        let area2 = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if area2 < 1e-15 {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        }

        let e12 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let e20 = [p0[0] - p2[0], p0[1] - p2[1], p0[2] - p2[2]];
        let cross = |normal: &[f64; 3], edge: &[f64; 3]| -> [f64; 3] {
            [
                normal[1] * edge[2] - normal[2] * edge[1],
                normal[2] * edge[0] - normal[0] * edge[2],
                normal[0] * edge[1] - normal[1] * edge[0],
            ]
        };

        let c0 = cross(&normal, &e12);
        let c1 = cross(&normal, &e20);
        let c2 = cross(&normal, &e01);
        let inv_2a = 1.0 / (area2 * area2);
        let grad = [
            inv_2a * (heat[i0] * c0[0] + heat[i1] * c1[0] + heat[i2] * c2[0]),
            inv_2a * (heat[i0] * c0[1] + heat[i1] * c1[1] + heat[i2] * c2[1]),
            inv_2a * (heat[i0] * c0[2] + heat[i1] * c1[2] + heat[i2] * c2[2]),
        ];

        let mag = (grad[0].powi(2) + grad[1].powi(2) + grad[2].powi(2)).sqrt();
        if mag < 1e-15 {
            face_grad.push([0.0, 0.0, 0.0]);
        } else {
            face_grad.push([-grad[0] / mag, -grad[1] / mag, -grad[2] / mag]);
        }
    }

    let mut div = vec![0.0f64; n];
    for (fi, cell) in faces.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let Some(ids) = valid_triangle_ids(cell, n) else {
            continue;
        };
        let pts: Vec<[f64; 3]> = ids.iter().map(|&idx| mesh.points.get(idx)).collect();
        let x = &face_grad[fi];

        for k in 0..3 {
            let k1 = (k + 1) % 3;
            let k2 = (k + 2) % 3;
            let e1 = [
                pts[k1][0] - pts[k][0],
                pts[k1][1] - pts[k][1],
                pts[k1][2] - pts[k][2],
            ];
            let e2 = [
                pts[k2][0] - pts[k][0],
                pts[k2][1] - pts[k][1],
                pts[k2][2] - pts[k][2],
            ];
            let dot_val = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
            let cross_mag = ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
            if cross_mag < 1e-15 {
                continue;
            }
            let cot = dot_val / cross_mag;
            let e_opp = [
                pts[k2][0] - pts[k1][0],
                pts[k2][1] - pts[k1][1],
                pts[k2][2] - pts[k1][2],
            ];
            let contrib = cot * (x[0] * e_opp[0] + x[1] * e_opp[1] + x[2] * e_opp[2]);
            div[ids[k]] += 0.5 * contrib;
        }
    }

    let poisson_iterations = 50;
    let mut phi = vec![0.0f64; n];
    for _ in 0..poisson_iterations {
        let mut new_phi = vec![0.0f64; n];
        for (i, nbrs) in neighbors.iter().enumerate() {
            if nbrs.is_empty() {
                new_phi[i] = phi[i];
                continue;
            }
            let count = nbrs.len() as f64;
            let mut sum = 0.0f64;
            for &nb in nbrs {
                sum += phi[nb];
            }
            new_phi[i] = (sum - div[i]) / count;
        }
        phi = new_phi;
    }

    let source_val = phi[source];
    let dist: Vec<f64> = phi.iter().map(|&v| (v - source_val).abs()).collect();

    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatGeodesic",
            dist,
            1,
        )));
    result.point_data_mut().set_active_scalars("HeatGeodesic");
    result
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

fn valid_triangle_ids(cell: &[i64], n_points: usize) -> Option<[usize; 3]> {
    Some([
        valid_point_id(*cell.first()?, n_points)?,
        valid_point_id(*cell.get(1)?, n_points)?,
        valid_point_id(*cell.get(2)?, n_points)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_heat_geo() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = heat_geodesic(&mesh, 0, 20);
        let arr = r.point_data().get_array("HeatGeodesic").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] < 0.01); // source should be near zero
    }
}
