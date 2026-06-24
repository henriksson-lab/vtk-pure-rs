//! Clip mesh by sphere (keep inside or outside).
use crate::data::{CellArray, Points, PolyData};
pub fn clip_inside_sphere(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    clip_sphere(mesh, center, radius, true)
}
pub fn clip_outside_sphere(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    clip_sphere(mesh, center, radius, false)
}
fn clip_sphere(mesh: &PolyData, center: [f64; 3], radius: f64, keep_inside: bool) -> PolyData {
    let r2 = radius * radius;
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
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
        let d2 = (cx - center[0]).powi(2) + (cy - center[1]).powi(2) + (cz - center[2]).powi(2);
        let inside = d2 <= r2;
        if inside == keep_inside {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        polys.push_cell(
            &cell
                .iter()
                .map(|&v| pt_map[v as usize] as i64)
                .collect::<Vec<_>>(),
        );
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inside() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_inside_sphere(&m, [0.0, 0.0, 0.0], 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_outside() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_outside_sphere(&m, [0.0, 0.0, 0.0], 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
