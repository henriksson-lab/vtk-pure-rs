//! Shrink each face toward its centroid.
use crate::data::{CellArray, Points, PolyData};
pub fn shrink_faces(mesh: &PolyData, factor: f64) -> PolyData {
    let f = factor.clamp(0.0, 1.0);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
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
        let base = pts.len();
        for &v in cell {
            let p = mesh.points.get(v as usize);
            pts.push([
                cx + (p[0] - cx) * f,
                cy + (p[1] - cy) * f,
                cz + (p[2] - cz) * f,
            ]);
        }
        let ids: Vec<i64> = (0..cell.len()).map(|i| (base + i) as i64).collect();
        polys.push_cell(&ids);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn explode_faces(mesh: &PolyData, distance: f64) -> PolyData {
    let mut gc = [0.0, 0.0, 0.0];
    let n = mesh.points.len();
    for i in 0..n {
        let p = mesh.points.get(i);
        gc[0] += p[0];
        gc[1] += p[1];
        gc[2] += p[2];
    }
    let nf = n as f64;
    gc[0] /= nf;
    gc[1] /= nf;
    gc[2] /= nf;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
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
        let cn = cell.len() as f64;
        cx /= cn;
        cy /= cn;
        cz /= cn;
        let dx = cx - gc[0];
        let dy = cy - gc[1];
        let dz = cz - gc[2];
        let dl = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
        let ox = dx / dl * distance;
        let oy = dy / dl * distance;
        let oz = dz / dl * distance;
        let base = pts.len();
        for &v in cell {
            let p = mesh.points.get(v as usize);
            pts.push([p[0] + ox, p[1] + oy, p[2] + oz]);
        }
        polys.push_cell(
            &(0..cell.len())
                .map(|i| (base + i) as i64)
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
    fn test_shrink() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [1.5, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = shrink_faces(&m, 0.5);
        assert_eq!(r.points.len(), 3);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_explode() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let r = explode_faces(&m, 0.5);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
