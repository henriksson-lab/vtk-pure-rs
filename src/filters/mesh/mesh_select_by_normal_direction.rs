//! Select faces by normal direction (facing up, down, etc).
use crate::data::{CellArray, Points, PolyData};
pub fn select_faces_facing(mesh: &PolyData, direction: [f64; 3], angle_threshold: f64) -> PolyData {
    let dl =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt()
            .max(1e-15);
    let d = [direction[0] / dl, direction[1] / dl, direction[2] / dl];
    let cos_t = angle_threshold.to_radians().cos();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nl < 1e-15 {
            continue;
        }
        let dot = (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / nl;
        if dot >= cos_t {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn select_upward_faces(mesh: &PolyData, angle: f64) -> PolyData {
    select_faces_facing(mesh, [0.0, 0.0, 1.0], angle)
}
pub fn select_downward_faces(mesh: &PolyData, angle: f64) -> PolyData {
    select_faces_facing(mesh, [0.0, 0.0, -1.0], angle)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_up() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, -1.0],
                [1.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = select_upward_faces(&m, 45.0);
        assert!(r.polys.num_cells() >= 1);
    }
}
