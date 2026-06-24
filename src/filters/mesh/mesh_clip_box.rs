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
        let inside = cx >= min[0]
            && cx <= max[0]
            && cy >= min[1]
            && cy <= max[1]
            && cz >= min[2]
            && cz <= max[2];
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
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
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
