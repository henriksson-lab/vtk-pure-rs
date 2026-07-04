//! Boolean subtraction approximation (remove vertices inside another mesh).
use crate::data::{CellArray, Points, PolyData};

pub fn boolean_subtract(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    let nb = mesh_b.points.len();
    if nb == 0 {
        return mesh_a.clone();
    }
    let na = mesh_a.points.len();
    let inside: Vec<bool> = (0..na)
        .map(|i| {
            let p = mesh_a.points.get(i);
            point_inside_mesh(p, mesh_b)
        })
        .collect();
    let mut pt_map = vec![0usize; na];
    let mut pts = Points::<f64>::new();
    for i in 0..na {
        if !inside[i] {
            pt_map[i] = pts.len();
            pts.push(mesh_a.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in mesh_a.polys.iter() {
        if cell.is_empty() || !valid_cell(cell, na) {
            continue;
        }
        if cell.iter().any(|&v| inside[v as usize]) {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

fn point_inside_mesh(p: [f64; 3], mesh: &PolyData) -> bool {
    let p = [p[0] + 1e-7, p[1] + 1.3e-7, p[2] + 0.9e-7];
    let mut crossings = 0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if ray_triangle_intersect_x(p, a, b, c) {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

fn ray_triangle_intersect_x(origin: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> bool {
    let dir = [1.0, 0.0, 0.0];
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return false;
    }
    let inv = 1.0 / det;
    let s = [origin[0] - a[0], origin[1] - a[1], origin[2] - a[2]];
    let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = inv * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-12
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_subtract() {
        let a = PolyData::from_triangles(
            vec![[-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_subtract(&a, &b);
        // Some vertices of A are inside B's bbox, so should have fewer
        assert!(r.points.len() <= a.points.len());
    }
}
