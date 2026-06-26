//! Vertex coloring utilities: procedural, from arrays, checkerboard patterns.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Color vertices by position using a checkerboard pattern.
pub fn checkerboard_color(mesh: &PolyData, cell_size: f64) -> PolyData {
    if !cell_size.is_finite() || cell_size <= 0.0 {
        return mesh.clone();
    }

    let n = mesh.points.len();
    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = mesh.points.get(i);
        let cx = (p[0] / cell_size).floor() as i64;
        let cy = (p[1] / cell_size).floor() as i64;
        let cz = (p[2] / cell_size).floor() as i64;
        let is_white = (cx + cy + cz) % 2 == 0;
        if is_white {
            rgb.extend_from_slice(&[1.0, 1.0, 1.0]);
        } else {
            rgb.extend_from_slice(&[0.2, 0.2, 0.2]);
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

/// Color vertices by their vertex index (rainbow gradient).
pub fn rainbow_by_index(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        let t = i as f64 / n.max(1) as f64;
        let (r, g, b) = hsv_to_rgb(t, 1.0, 1.0);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

/// Color by curvature magnitude (blue=flat, red=curved).
pub fn color_by_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut curvature = Vec::with_capacity(n);
    for i in 0..n {
        if adj[i].is_empty() {
            curvature.push(0.0);
            continue;
        }
        let p = mesh.points.get(i);
        let mut lap = [0.0; 3];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            for c in 0..3 {
                lap[c] += q[c] - p[c];
            }
        }
        let k = adj[i].len() as f64;
        curvature.push(((lap[0] / k).powi(2) + (lap[1] / k).powi(2) + (lap[2] / k).powi(2)).sqrt());
    }

    let max_c = curvature.iter().cloned().fold(0.0f64, f64::max).max(1e-15);
    let mut rgb = Vec::with_capacity(n * 3);
    for &c in &curvature {
        let t = (c / max_c).clamp(0.0, 1.0);
        rgb.push(t);
        rgb.push(0.0);
        rgb.push(1.0 - t);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Curvature",
            curvature,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    result
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let x = cell[i] as usize;
            let y = cell[(i + 1) % nc] as usize;
            if x < n && y < n {
                adj[x].insert(y);
                adj[y].insert(x);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checker() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.5, 0.0, 0.0], [0.0, 1.5, 0.0]]);
        let result = checkerboard_color(&mesh, 1.0);
        assert!(result.point_data().get_array("RGB").is_some());
    }

    #[test]
    fn checker_invalid_cell_size_is_noop() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let result = checkerboard_color(&mesh, 0.0);
        assert!(result.point_data().get_array("RGB").is_none());
    }

    #[test]
    fn rainbow() {
        let mesh = PolyData::from_points(vec![[0.0; 3]; 5]);
        let result = rainbow_by_index(&mesh);
        let arr = result.point_data().get_array("RGB").unwrap();
        assert_eq!(arr.num_components(), 3);
    }
    #[test]
    fn curv_color() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = color_by_curvature(&mesh);
        assert!(result.point_data().get_array("Curvature").is_some());
        assert!(result.point_data().get_array("RGB").is_some());
    }
}
