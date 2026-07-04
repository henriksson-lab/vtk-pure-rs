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
    let kept_verts = collect_kept_cells(&mesh.verts, mesh, center, r2, keep_inside, &mut used);
    let kept_lines = collect_kept_cells(&mesh.lines, mesh, center, r2, keep_inside, &mut used);
    let kept_polys = collect_kept_cells(&mesh.polys, mesh, center, r2, keep_inside, &mut used);
    let kept_strips = collect_kept_cells(&mesh.strips, mesh, center, r2, keep_inside, &mut used);

    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = remap_cells(&kept_verts, &pt_map);
    r.lines = remap_cells(&kept_lines, &pt_map);
    r.polys = remap_cells(&kept_polys, &pt_map);
    r.strips = remap_cells(&kept_strips, &pt_map);
    r
}

fn collect_kept_cells(
    cells: &CellArray,
    mesh: &PolyData,
    center: [f64; 3],
    r2: f64,
    keep_inside: bool,
    used: &mut [bool],
) -> Vec<Vec<i64>> {
    let mut kept = Vec::new();
    for cell in cells.iter() {
        let Some([cx, cy, cz]) = cell_centroid(mesh, cell) else {
            continue;
        };
        let d2 = (cx - center[0]).powi(2) + (cy - center[1]).powi(2) + (cz - center[2]).powi(2);
        let inside = d2 <= r2;
        if inside == keep_inside {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    kept
}

fn remap_cells(cells: &[Vec<i64>], pt_map: &[usize]) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        out.push_cell(&mapped);
    }
    out
}

fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    if cell.is_empty()
        || cell
            .iter()
            .any(|&v| v < 0 || v as usize >= mesh.points.len())
    {
        return None;
    }
    let mut c = [0.0; 3];
    for &v in cell {
        let p = mesh.points.get(v as usize);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = cell.len() as f64;
    Some([c[0] / n, c[1] / n, c[2] / n])
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
