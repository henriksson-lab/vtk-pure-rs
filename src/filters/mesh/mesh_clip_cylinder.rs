//! Clip mesh by infinite cylinder.
use crate::data::{CellArray, Points, PolyData};
pub fn clip_inside_cylinder(
    mesh: &PolyData,
    axis_origin: [f64; 3],
    axis_dir: [f64; 3],
    radius: f64,
) -> PolyData {
    clip_cyl(mesh, axis_origin, axis_dir, radius, true)
}
pub fn clip_outside_cylinder(
    mesh: &PolyData,
    axis_origin: [f64; 3],
    axis_dir: [f64; 3],
    radius: f64,
) -> PolyData {
    clip_cyl(mesh, axis_origin, axis_dir, radius, false)
}
fn clip_cyl(mesh: &PolyData, o: [f64; 3], d: [f64; 3], r: f64, inside: bool) -> PolyData {
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
    let dn = [d[0] / dl, d[1] / dl, d[2] / dl];
    let r2 = r * r;
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
        let v = [cx - o[0], cy - o[1], cz - o[2]];
        let proj = v[0] * dn[0] + v[1] * dn[1] + v[2] * dn[2];
        let perp = [
            v[0] - proj * dn[0],
            v[1] - proj * dn[1],
            v[2] - proj * dn[2],
        ];
        let d2 = perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2];
        if (d2 <= r2) == inside {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [0.25, 0.5, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_inside_cylinder(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
