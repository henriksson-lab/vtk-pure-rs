use std::collections::HashSet;

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Geodesic distance using a simplified heat method.
///
/// The algorithm:
/// 1. Place a heat impulse at the source vertex and diffuse using explicit
///    integration for a number of steps determined by `heat_time`.
/// 2. Compute the gradient of the heat field on each triangle and normalize.
/// 3. Solve for the distance by integrating the divergence of the normalized
///    gradient (simplified: iterative Jacobi on the Poisson equation).
///
/// The result is an approximate geodesic distance stored as "GeodesicDistance"
/// point data on the returned PolyData.
pub fn geodesic_distance_heat(input: &PolyData, source: usize, heat_time: f64) -> PolyData {
    let n = input.points.len();
    let mut output = input.clone();
    if n == 0 || source >= n {
        return output;
    }

    // Build adjacency
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            if a >= n || b >= n {
                continue;
            }
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    // Compute average edge length for time step
    let mut avg_edge: f64 = 0.0;
    let mut edge_count: f64 = 0.0;
    for (i, nbrs) in neighbors.iter().enumerate() {
        let pi = input.points.get(i);
        for &nb in nbrs {
            if nb > i {
                let pj = input.points.get(nb);
                let d: f64 =
                    ((pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2))
                        .sqrt();
                avg_edge += d;
                edge_count += 1.0;
            }
        }
    }
    if edge_count < 1.0 {
        let dist = vec![0.0f64; n];
        output
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "GeodesicDistance",
                dist,
                1,
            )));
        return output;
    }
    avg_edge /= edge_count;

    let dt: f64 = heat_time * avg_edge * avg_edge;
    let num_diffusion_steps: usize = 20;

    // Step 1: Diffuse heat from source
    let mut heat = vec![0.0f64; n];
    heat[source] = 1.0;

    for _ in 0..num_diffusion_steps {
        let mut new_heat = vec![0.0f64; n];
        for (i, nbrs) in neighbors.iter().enumerate() {
            if nbrs.is_empty() {
                new_heat[i] = heat[i];
                continue;
            }
            let count: f64 = nbrs.len() as f64;
            let mut avg: f64 = 0.0;
            for &nb in nbrs {
                avg += heat[nb];
            }
            avg /= count;
            new_heat[i] = heat[i] + dt * (avg - heat[i]);
        }
        heat = new_heat;
    }

    // Step 2: Compute gradient per triangle and normalize
    let num_faces = input.polys.num_cells();
    let mut face_grad: Vec<[f64; 3]> = Vec::with_capacity(num_faces);
    let faces: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    for cell in &faces {
        if cell.len() < 3 {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        }
        let i0 = cell[0] as usize;
        let i1 = cell[1] as usize;
        let i2 = cell[2] as usize;
        if i0 >= n || i1 >= n || i2 >= n {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        }

        let p0 = input.points.get(i0);
        let p1 = input.points.get(i1);
        let p2 = input.points.get(i2);

        // Edge vectors
        let e01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        // Face normal (unnormalized)
        let normal = [
            e01[1] * e02[2] - e01[2] * e02[1],
            e01[2] * e02[0] - e01[0] * e02[2],
            e01[0] * e02[1] - e01[1] * e02[0],
        ];

        // Gradient of a linear function on a triangle: grad u = (1/2A) * sum( u_i * (N x e_opp_i) )
        let area2: f64 = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if area2 < 1e-15 {
            face_grad.push([0.0, 0.0, 0.0]);
            continue;
        }

        // Opposite edges
        let e12 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let e20 = [p0[0] - p2[0], p0[1] - p2[1], p0[2] - p2[2]];

        // N x e for each vertex
        let cross = |n: &[f64; 3], e: &[f64; 3]| -> [f64; 3] {
            [
                n[1] * e[2] - n[2] * e[1],
                n[2] * e[0] - n[0] * e[2],
                n[0] * e[1] - n[1] * e[0],
            ]
        };

        let c0 = cross(&normal, &e12);
        let c1 = cross(&normal, &e20);
        let c2 = cross(&normal, &e01);

        let inv_2a: f64 = 1.0 / (area2 * area2); // 1/(2A) where area2 = 2A already
        let grad = [
            inv_2a * (heat[i0] * c0[0] + heat[i1] * c1[0] + heat[i2] * c2[0]),
            inv_2a * (heat[i0] * c0[1] + heat[i1] * c1[1] + heat[i2] * c2[1]),
            inv_2a * (heat[i0] * c0[2] + heat[i1] * c1[2] + heat[i2] * c2[2]),
        ];

        // Normalize (negate to point from high to low)
        let mag: f64 = (grad[0].powi(2) + grad[1].powi(2) + grad[2].powi(2)).sqrt();
        if mag < 1e-15 {
            face_grad.push([0.0, 0.0, 0.0]);
        } else {
            face_grad.push([-grad[0] / mag, -grad[1] / mag, -grad[2] / mag]);
        }
    }

    // Step 3: Compute divergence at vertices and solve Poisson with Jacobi iteration
    // Accumulate integrated divergence per vertex
    let mut div = vec![0.0f64; n];
    for (fi, cell) in faces.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        if ids.iter().any(|&id| id >= n) {
            continue;
        }
        let pts: Vec<[f64; 3]> = ids.iter().map(|&idx| input.points.get(idx)).collect();
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
            // Cotangent weights (simplified)
            let dot_val: f64 = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
            let cross_mag: f64 = ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
            if cross_mag < 1e-15 {
                continue;
            }
            let cot: f64 = dot_val / cross_mag;

            let e_opp = [
                pts[k2][0] - pts[k1][0],
                pts[k2][1] - pts[k1][1],
                pts[k2][2] - pts[k1][2],
            ];
            let contrib: f64 = cot * (x[0] * e_opp[0] + x[1] * e_opp[1] + x[2] * e_opp[2]);
            div[ids[k]] += 0.5 * contrib;
        }
    }

    // Jacobi iterations for Poisson equation: L * phi = div
    let poisson_iterations: usize = 50;
    let mut phi = vec![0.0f64; n];
    for _ in 0..poisson_iterations {
        let mut new_phi = vec![0.0f64; n];
        for (i, nbrs) in neighbors.iter().enumerate() {
            if nbrs.is_empty() {
                new_phi[i] = phi[i];
                continue;
            }
            let count: f64 = nbrs.len() as f64;
            let mut sum: f64 = 0.0;
            for &nb in nbrs {
                sum += phi[nb];
            }
            new_phi[i] = (sum - div[i]) / count;
        }
        phi = new_phi;
    }

    // Shift so source = 0
    let source_val: f64 = phi[source];
    let dist: Vec<f64> = phi.iter().map(|&v| (v - source_val).abs()).collect();

    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist,
            1,
        )));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CellArray;

    fn make_line_strip_mesh() -> PolyData {
        // A flat strip of triangles: 0-1-2-3-4 along x-axis
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
            pd.points.push([i as f64, 1.0, 0.0]);
        }
        let mut polys = CellArray::new();
        for i in 0..4u32 {
            let bl = (i * 2) as i64;
            let br = (i * 2 + 2) as i64;
            let tl = (i * 2 + 1) as i64;
            let tr = (i * 2 + 3) as i64;
            polys.push_cell(&[bl, br, tl]);
            polys.push_cell(&[br, tr, tl]);
        }
        pd.polys = polys;
        pd
    }

    #[test]
    fn source_has_zero_distance() {
        let input = make_line_strip_mesh();
        let result = geodesic_distance_heat(&input, 0, 1.0);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].abs() < 1e-10, "source distance should be zero");
    }

    #[test]
    fn distance_increases_from_source() {
        let input = make_line_strip_mesh();
        let result = geodesic_distance_heat(&input, 0, 1.0);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        // Points at x=0 (indices 0,1) should be nearer than x=2 (indices 4,5)
        arr.tuple_as_f64(0, &mut buf);
        let d0: f64 = buf[0];
        arr.tuple_as_f64(4, &mut buf);
        let d2: f64 = buf[0];
        assert!(
            d2 > d0,
            "distance at x=2 ({}) should exceed distance at x=0 ({})",
            d2,
            d0
        );
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let input = PolyData::new();
        let result = geodesic_distance_heat(&input, 0, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
