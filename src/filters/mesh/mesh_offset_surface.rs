//! Create offset surface along vertex normals.
use crate::data::{CellArray, Points, PolyData};
pub fn offset_surface(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_normals(mesh);
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * distance,
            p[1] + nm[i][1] * distance,
            p[2] + nm[i][2] * distance,
        ]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = mesh.polys.clone();
    r
}
pub fn shell(mesh: &PolyData, inner_offset: f64, outer_offset: f64) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_normals(mesh);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * inner_offset,
            p[1] + nm[i][1] * inner_offset,
            p[2] + nm[i][2] * inner_offset,
        ]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * outer_offset,
            p[1] + nm[i][1] * outer_offset,
            p[2] + nm[i][2] * outer_offset,
        ]);
    }
    // Inner surface (reversed)
    for cell in mesh.polys.iter() {
        let mut rev: Vec<i64> = cell.to_vec();
        rev.reverse();
        polys.push_cell(&rev);
    }
    // Outer surface
    for cell in mesh.polys.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        polys.push_cell(&shifted);
    }
    // Side walls on boundary edges
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for (&(a, b), &c) in &ec {
        if c == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
fn calc_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_offset() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = offset_surface(&m, 0.1);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_shell() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = shell(&m, -0.05, 0.05);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() > 2);
    }
}
