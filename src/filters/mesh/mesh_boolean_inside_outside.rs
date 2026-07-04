//! Classify mesh faces as inside/outside another mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_faces_inside(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() || !valid_cell(cell, mesh.points.len()) {
            data.push(0.0);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &v in cell {
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;
        let inside = point_in_mesh([cx, cy, cz], reference);
        data.push(if inside { 1.0 } else { 0.0 });
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Inside", data, 1)));
    r
}
fn point_in_mesh(p: [f64; 3], mesh: &PolyData) -> bool {
    let Some(bounds) = mesh_bounds(mesh) else {
        return false;
    };
    if p[0] < bounds[0]
        || p[0] > bounds[1]
        || p[1] < bounds[2]
        || p[1] > bounds[3]
        || p[2] < bounds[4]
        || p[2] > bounds[5]
    {
        return false;
    }

    const RAYS: [[f64; 3]; 10] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.5],
        [-0.5, 1.0, 1.0],
        [1.0, -0.5, 1.0],
        [-1.0, 0.25, 0.5],
        [0.25, -1.0, 0.5],
        [0.5, 0.25, -1.0],
        [1.0, 0.5, -0.25],
    ];
    let mut delta_votes: i32 = 0;
    for ray in RAYS {
        let num_ints = count_ray_intersections(p, normalize(ray), mesh);
        if num_ints % 2 == 0 {
            delta_votes -= 1;
        } else {
            delta_votes += 1;
        }
        if delta_votes.abs() >= 2 {
            break;
        }
    }
    delta_votes >= 0
}

fn count_ray_intersections(p: [f64; 3], d: [f64; 3], mesh: &PolyData) -> usize {
    let mut intersections = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if let Some(t) = ray_tri(p, d, a, b, c) {
                intersections.push(t);
            }
        }
    }
    count_unique_intersections(intersections)
}

fn ray_tri(o: [f64; 3], d: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = [o[0] - a[0], o[1] - a[1], o[2] - a[2]];
    let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = inv * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    (t > 1e-12).then_some(t)
}

fn count_unique_intersections(mut intersections: Vec<f64>) -> usize {
    intersections.sort_by(|a, b| a.total_cmp(b));
    let mut count = 0;
    let mut last = None;
    for t in intersections {
        if last.is_none_or(|prev| t - prev > 1e-7) {
            count += 1;
            last = Some(t);
        }
    }
    count
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn mesh_bounds(mesh: &PolyData) -> Option<[f64; 6]> {
    if mesh.points.is_empty() {
        return None;
    }
    let p0 = mesh.points.get(0);
    let mut bounds = [p0[0], p0[0], p0[1], p0[1], p0[2], p0[2]];
    for i in 1..mesh.points.len() {
        let p = mesh.points.get(i);
        bounds[0] = bounds[0].min(p[0]);
        bounds[1] = bounds[1].max(p[0]);
        bounds[2] = bounds[2].min(p[1]);
        bounds[3] = bounds[3].max(p[1]);
        bounds[4] = bounds[4].min(p[2]);
        bounds[5] = bounds[5].max(p[2]);
    }
    Some(bounds)
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let inner = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let outer = PolyData::from_triangles(
            vec![
                [-5.0, -5.0, -5.0],
                [5.0, -5.0, -5.0],
                [0.0, 5.0, -5.0],
                [0.0, 0.0, 5.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = classify_faces_inside(&inner, &outer);
        assert!(r.cell_data().get_array("Inside").is_some());
    }
}
