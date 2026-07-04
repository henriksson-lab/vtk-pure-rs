//! Crop mesh to one half along an axis.
use crate::data::{CellArray, Points, PolyData};
pub fn crop_positive_x(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 0, true)
}
pub fn crop_negative_x(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 0, false)
}
pub fn crop_positive_y(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 1, true)
}
pub fn crop_negative_y(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 1, false)
}
pub fn crop_positive_z(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 2, true)
}
pub fn crop_negative_z(mesh: &PolyData) -> PolyData {
    crop_axis(mesh, 2, false)
}
fn crop_axis(mesh: &PolyData, axis: usize, positive: bool) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    for i in 0..n {
        cx += mesh.points.get(i)[axis];
    }
    cx /= n as f64;
    let mut used = vec![false; n];
    let kept_verts = collect_kept_cells(&mesh.verts, mesh, axis, positive, cx, &mut used);
    let kept_lines = collect_kept_cells(&mesh.lines, mesh, axis, positive, cx, &mut used);
    let kept_polys = collect_kept_cells(&mesh.polys, mesh, axis, positive, cx, &mut used);
    let kept_strips = collect_kept_cells(&mesh.strips, mesh, axis, positive, cx, &mut used);
    let mut pm = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = remap_cells(&kept_verts, &pm);
    r.lines = remap_cells(&kept_lines, &pm);
    r.polys = remap_cells(&kept_polys, &pm);
    r.strips = remap_cells(&kept_strips, &pm);
    r
}

fn collect_kept_cells(
    cells: &CellArray,
    mesh: &PolyData,
    axis: usize,
    positive: bool,
    center: f64,
    used: &mut [bool],
) -> Vec<Vec<i64>> {
    let mut kept = Vec::new();
    for cell in cells.iter() {
        if cell.is_empty() {
            continue;
        }
        if cell
            .iter()
            .any(|&v| v < 0 || v as usize >= mesh.points.len())
        {
            continue;
        }
        let mut avg = 0.0;
        for &v in cell {
            avg += mesh.points.get(v as usize)[axis];
        }
        avg /= cell.len() as f64;
        let keep = if positive {
            avg >= center
        } else {
            avg <= center
        };
        if keep {
            for &v in cell {
                if v >= 0 && (v as usize) < used.len() {
                    used[v as usize] = true;
                }
            }
            kept.push(cell.to_vec());
        }
    }
    kept
}

fn remap_cells(cells: &[Vec<i64>], point_map: &[usize]) -> CellArray {
    let mut out = CellArray::new();
    for c in cells {
        out.push_cell(
            &c.iter()
                .map(|&v| point_map[v as usize] as i64)
                .collect::<Vec<_>>(),
        );
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [-0.5, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let r = crop_positive_x(&m);
        assert!(r.polys.num_cells() >= 1);
    }
}
