//! Clip mesh by axis-aligned bounding box.

use crate::data::{CellArray, Points, PolyData};

/// Keep only cells whose centroid is inside the given AABB.
pub fn clip_by_box(mesh: &PolyData, min: [f64; 3], max: [f64; 3]) -> PolyData {
    clip_impl(mesh, min, max, true)
}

/// Keep only cells whose centroid is outside the given AABB.
pub fn clip_outside_box(mesh: &PolyData, min: [f64; 3], max: [f64; 3]) -> PolyData {
    clip_impl(mesh, min, max, false)
}

/// Keep only vertices inside the box (as point cloud).
pub fn clip_points_by_box(mesh: &PolyData, min: [f64; 3], max: [f64; 3]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        if p[0] >= min[0]
            && p[0] <= max[0]
            && p[1] >= min[1]
            && p[1] <= max[1]
            && p[2] >= min[2]
            && p[2] <= max[2]
        {
            let idx = pts.len();
            pts.push(p);
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

fn clip_impl(mesh: &PolyData, min: [f64; 3], max: [f64; 3], keep_inside: bool) -> PolyData {
    let mut used = vec![false; mesh.points.len()];
    let kept_verts = collect_kept_cells(&mesh.verts, mesh, min, max, keep_inside, &mut used);
    let kept_lines = collect_kept_cells(&mesh.lines, mesh, min, max, keep_inside, &mut used);
    let kept_polys = collect_kept_cells(&mesh.polys, mesh, min, max, keep_inside, &mut used);
    let kept_strips = collect_kept_cells(&mesh.strips, mesh, min, max, keep_inside, &mut used);

    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = remap_cells(&kept_verts, &pt_map);
    result.lines = remap_cells(&kept_lines, &pt_map);
    result.polys = remap_cells(&kept_polys, &pt_map);
    result.strips = remap_cells(&kept_strips, &pt_map);
    result
}

fn collect_kept_cells(
    cells: &CellArray,
    mesh: &PolyData,
    min: [f64; 3],
    max: [f64; 3],
    keep_inside: bool,
    used: &mut [bool],
) -> Vec<Vec<i64>> {
    let mut kept = Vec::new();
    for cell in cells.iter() {
        let Some(center) = cell_centroid(mesh, cell) else {
            continue;
        };
        let inside = center[0] >= min[0]
            && center[0] <= max[0]
            && center[1] >= min[1]
            && center[1] <= max[1]
            && center[2] >= min[2]
            && center[2] <= max[2];
        if inside == keep_inside {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    kept
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

fn remap_cells(cells: &[Vec<i64>], pt_map: &[usize]) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        out.push_cell(&mapped);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clip_inside() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [5.0, 5.0, 5.0],
                [6.0, 5.0, 5.0],
                [5.5, 6.0, 5.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_by_box(&mesh, [-1.0, -1.0, -1.0], [2.0, 2.0, 2.0]);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_clip_outside() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [5.0, 5.0, 5.0],
                [6.0, 5.0, 5.0],
                [5.5, 6.0, 5.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_outside_box(&mesh, [-1.0, -1.0, -1.0], [2.0, 2.0, 2.0]);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_points() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([5.0, 5.0, 5.0]);
        mesh.points.push([10.0, 10.0, 10.0]);
        let r = clip_points_by_box(&mesh, [-1.0, -1.0, -1.0], [6.0, 6.0, 6.0]);
        assert_eq!(r.points.len(), 2);
    }
}
