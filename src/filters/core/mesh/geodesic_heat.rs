//! Geodesic distance via heat method (fast approximate geodesics).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute approximate geodesic distance from seed vertices using the heat method.
///
/// Steps: 1) Solve heat equation from seeds, 2) Compute gradient, 3) Solve Poisson.
/// This implementation uses iterative Jacobi relaxation as an approximation.
pub fn geodesic_heat(mesh: &PolyData, seeds: &[usize], iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);

    // Step 1: Heat diffusion from seeds
    let mut heat = vec![0.0f64; n];
    for &s in seeds {
        if s < n {
            heat[s] = 1.0;
        }
    }
    let avg_edge = avg_edge_len(mesh, &adj);
    let dt = avg_edge * avg_edge;

    for _ in 0..iterations {
        let mut new_heat = heat.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut lap = 0.0;
            for &j in &adj[i] {
                lap += heat[j] - heat[i];
            }
            new_heat[i] = heat[i] + dt * lap / adj[i].len() as f64;
        }
        heat = new_heat;
    }

    // Step 2: Normalize gradient and solve for distance
    // Approximate: distance ∝ -log(heat) for diffusion kernel
    let max_heat = heat.iter().cloned().fold(0.0f64, f64::max).max(1e-15);
    let mut dist: Vec<f64> = heat
        .iter()
        .map(|&h| {
            if h > 1e-20 {
                -(h / max_heat).ln() * avg_edge
            } else {
                f64::MAX
            }
        })
        .collect();

    // Normalize
    let max_dist = dist
        .iter()
        .filter(|&&d| d < f64::MAX)
        .cloned()
        .fold(0.0f64, f64::max)
        .max(1e-15);
    for d in &mut dist {
        if *d >= f64::MAX {
            *d = max_dist;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicHeat",
            dist,
            1,
        )));
    result
}

/// Compute geodesic iso-contours at regular distance intervals.
pub fn geodesic_iso_lines(
    mesh: &PolyData,
    seeds: &[usize],
    n_contours: usize,
    iterations: usize,
) -> PolyData {
    let with_dist = geodesic_heat(mesh, seeds, iterations);
    let arr = match with_dist.point_data().get_array("GeodesicHeat") {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let mut buf = [0.0f64];
    let mut max_d = 0.0f64;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        max_d = max_d.max(buf[0]);
    }
    if max_d < 1e-15 {
        return PolyData::new();
    }

    let mut isovalues = Vec::with_capacity(n_contours);
    for i in 1..=n_contours {
        isovalues.push(max_d * i as f64 / (n_contours + 1) as f64);
    }

    // Extract contours using the distance field
    let scalars: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut all_points = crate::data::Points::<f64>::new();
    let mut all_lines = crate::data::CellArray::new();

    for &iso in &isovalues {
        for cell in with_dist.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            if cell
                .iter()
                .any(|&id| id < 0 || id as usize >= scalars.len())
            {
                continue;
            }
            let mut crossings = Vec::new();
            for edge_i in 0..cell.len() {
                let a = cell[edge_i] as usize;
                let b = cell[(edge_i + 1) % cell.len()] as usize;
                let va = scalars[a];
                let vb = scalars[b];
                if (va - iso) * (vb - iso) >= 0.0 {
                    continue;
                }
                let t = (iso - va) / (vb - va);
                let pa = with_dist.points.get(a);
                let pb = with_dist.points.get(b);
                let pt = [
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ];
                let idx = all_points.len() as i64;
                all_points.push(pt);
                crossings.push(idx);
            }
            for pair in crossings.chunks_exact(2) {
                all_lines.push_cell(pair);
            }
        }
    }

    let mut result = PolyData::new();
    result.points = all_points;
    result.lines = all_lines;
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

fn avg_edge_len(mesh: &PolyData, adj: &[Vec<usize>]) -> f64 {
    let mut total = 0.0;
    let mut count = 0;
    for (i, nbs) in adj.iter().enumerate() {
        for &j in nbs {
            if j > i {
                let a = mesh.points.get(i);
                let b = mesh.points.get(j);
                total +=
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                count += 1;
            }
        }
    }
    if count > 0 {
        total / count as f64
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heat_distance() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = geodesic_heat(&mesh, &[0], 50);
        let arr = result.point_data().get_array("GeodesicHeat").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let d0 = buf[0];
        arr.tuple_as_f64(99, &mut buf);
        let d99 = buf[0];
        assert!(d99 > d0, "far corner should have larger distance");
    }
    #[test]
    fn iso_lines() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..7 {
            for x in 0..7 {
                let bl = y * 8 + x;
                tris.push([bl, bl + 1, bl + 9]);
                tris.push([bl, bl + 9, bl + 8]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let lines = geodesic_iso_lines(&mesh, &[0], 3, 30);
        assert!(lines.points.len() > 0);
    }

    #[test]
    fn iso_lines_pair_crossings_per_cell() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([10.0, 0.0, 0.0]);
        mesh.points.push([11.0, 0.0, 0.0]);
        mesh.points.push([10.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[3, 4, 5]);

        let lines = geodesic_iso_lines(&mesh, &[0], 1, 1);
        for cell in lines.lines.iter() {
            assert_eq!(cell.len(), 2);
            let a = lines.points.get(cell[0] as usize);
            let b = lines.points.get(cell[1] as usize);
            assert!(
                (a[0] - b[0]).abs() < 2.0,
                "iso segment endpoints should come from the same source cell"
            );
        }
    }

    #[test]
    fn iso_lines_skips_invalid_cell_ids() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[0, 2, 99]);

        let lines = geodesic_iso_lines(&mesh, &[0], 1, 1);
        assert!(lines.points.len() <= 2);
    }
}
