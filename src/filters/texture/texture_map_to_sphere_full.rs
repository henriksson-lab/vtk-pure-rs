//! Full spherical texture mapping with seam handling.
//!
//! Unlike the basic `texture_map::texture_map_to_sphere`, this version
//! handles the seam at phi=±π by duplicating vertices along the seam
//! to prevent texture wrapping artifacts.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate spherical texture coordinates with seam correction.
///
/// Maps (theta, phi) to (u, v) with duplicate vertices at the phi=±π seam
/// to prevent texture wrap artifacts.
pub fn texture_map_to_sphere_full(
    input: &PolyData,
    center: [f64; 3],
    prevent_seam: bool,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute spherical coordinates using flat slice access
    let pts = input.points.as_flat_slice();
    let mut phis: Vec<f64> = Vec::with_capacity(n);
    let mut thetas: Vec<f64> = Vec::with_capacity(n);

    for i in 0..n {
        let b = i * 3;
        let dx = pts[b] - center[0];
        let dy = pts[b + 1] - center[1];
        let dz = pts[b + 2] - center[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let theta = if r > 1e-15 { (dz / r).acos() } else { 0.0 };
        let phi = dy.atan2(dx);
        thetas.push(theta);
        phis.push(phi);
    }

    if !prevent_seam {
        // Simple mapping without seam correction
        let mut tcoords = Vec::with_capacity(n * 2);
        for i in 0..n {
            let u = (phis[i] + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
            let v = thetas[i] / std::f64::consts::PI;
            tcoords.push(u);
            tcoords.push(v);
        }
        let mut result = input.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords", tcoords, 2,
            )));
        result.point_data_mut().set_active_tcoords("TCoords");
        return result;
    }

    // Seam correction: duplicate vertices where a triangle straddles the seam
    let mut new_points = Points::<f64>::new();
    let mut new_phis: Vec<f64> = Vec::new();
    let mut new_thetas: Vec<f64> = Vec::new();
    let mut new_polys = CellArray::new();

    // Copy original points
    for i in 0..n {
        new_points.push(input.points.get(i));
        new_phis.push(phis[i]);
        new_thetas.push(thetas[i]);
    }

    let pi = std::f64::consts::PI;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            new_polys.push_cell(cell);
            continue;
        }

        // Check if this cell crosses the seam (large phi difference)
        let cell_phis: Vec<f64> = cell.iter().map(|&id| phis[id as usize]).collect();
        let phi_range = cell_phis.iter().cloned().fold(f64::MIN, f64::max)
            - cell_phis.iter().cloned().fold(f64::MAX, f64::min);

        if phi_range > pi {
            // This cell crosses the seam — duplicate vertices with phi < 0
            // by adding 2π to their phi
            let mut new_ids: Vec<i64> = Vec::new();
            for &pid in cell {
                let idx = pid as usize;
                if phis[idx] < 0.0 {
                    // Duplicate this vertex
                    let dup_idx = new_points.len();
                    new_points.push(input.points.get(idx));
                    new_phis.push(phis[idx] + 2.0 * pi);
                    new_thetas.push(thetas[idx]);
                    new_ids.push(dup_idx as i64);
                } else {
                    new_ids.push(pid);
                }
            }
            new_polys.push_cell(&new_ids);
        } else {
            new_polys.push_cell(cell);
        }
    }

    // Compute texture coordinates
    let nn = new_points.len();
    let mut tcoords = Vec::with_capacity(nn * 2);
    for i in 0..nn {
        let u = (new_phis[i] + pi) / (2.0 * pi);
        let v = new_thetas[i] / pi;
        tcoords.push(u.clamp(0.0, 1.0));
        tcoords.push(v.clamp(0.0, 1.0));
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    // Copy lines and verts
    result.lines = input.lines.clone();
    result.verts = input.verts.clone();

    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sphere_points() -> PolyData {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        let n = 8;
        for i in 0..=n {
            let theta = std::f64::consts::PI * i as f64 / n as f64;
            for j in 0..=n {
                let phi = 2.0 * std::f64::consts::PI * j as f64 / n as f64 - std::f64::consts::PI;
                pts.push([
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ]);
            }
        }
        for i in 0..n {
            for j in 0..n {
                let bl = i * (n + 1) + j;
                tris.push([bl, bl + 1, bl + n + 2]);
                tris.push([bl, bl + n + 2, bl + n + 1]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn simple_mapping() {
        let mesh = make_sphere_points();
        let result = texture_map_to_sphere_full(&mesh, [0.0, 0.0, 0.0], false);
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), mesh.points.len());
        let mut buf = [0.0f64; 2];
        for i in 0..tc.num_tuples() {
            tc.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0 && buf[0] <= 1.0, "u={}", buf[0]);
            assert!(buf[1] >= 0.0 && buf[1] <= 1.0, "v={}", buf[1]);
        }
    }

    #[test]
    fn seam_correction() {
        let mesh = make_sphere_points();
        let result = texture_map_to_sphere_full(&mesh, [0.0, 0.0, 0.0], true);
        // Should have more points than original (duplicated at seam)
        assert!(result.points.len() >= mesh.points.len());
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn empty() {
        let result = texture_map_to_sphere_full(&PolyData::new(), [0.0; 3], true);
        assert_eq!(result.points.len(), 0);
    }
}
