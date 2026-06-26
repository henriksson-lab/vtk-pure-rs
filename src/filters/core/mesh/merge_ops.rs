//! Mesh merge operations: append, zip, interleave.

use crate::data::{CellArray, Points, PolyData};

/// Append multiple meshes into one (no point merging).
pub fn append_meshes(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    if meshes.len() == 1 {
        return meshes[0].clone();
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    for &m in meshes {
        let base = pts.len() as i64;
        for i in 0..m.points.len() {
            pts.push(m.points.get(i));
        }
        copy_cells(&m.verts, &mut verts, base);
        copy_cells(&m.lines, &mut lines, base);
        copy_cells(&m.polys, &mut polys, base);
        copy_cells(&m.strips, &mut strips, base);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r.lines = lines;
    r.polys = polys;
    r.strips = strips;
    r
}

/// Zip two meshes: interleave their triangles.
pub fn zip_meshes(a: &PolyData, b: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Copy A's points
    let offset_a = 0i64;
    for i in 0..a.points.len() {
        pts.push(a.points.get(i));
    }
    let offset_b = a.points.len() as i64;
    for i in 0..b.points.len() {
        pts.push(b.points.get(i));
    }

    let a_cells: Vec<Vec<i64>> = a.polys.iter().map(|c| c.to_vec()).collect();
    let b_cells: Vec<Vec<i64>> = b
        .polys
        .iter()
        .map(|c| c.iter().map(|&id| id + offset_b).collect())
        .collect();

    let max_len = a_cells.len().max(b_cells.len());
    for i in 0..max_len {
        if i < a_cells.len() {
            polys.push_cell(&a_cells[i]);
        }
        if i < b_cells.len() {
            polys.push_cell(&b_cells[i]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Stack meshes vertically (translate each by cumulative height along Y).
pub fn stack_vertical(meshes: &[&PolyData], gap: f64) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    let mut y_offset = 0.0;

    for mesh in meshes {
        let base = pts.len() as i64;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            min_y = min_y.min(p[1]);
            max_y = max_y.max(p[1]);
        }
        let translate_y = if mesh.points.is_empty() {
            y_offset
        } else {
            y_offset - min_y
        };
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            pts.push([p[0], p[1] + translate_y, p[2]]);
        }
        copy_cells(&mesh.verts, &mut verts, base);
        copy_cells(&mesh.lines, &mut lines, base);
        copy_cells(&mesh.polys, &mut polys, base);
        copy_cells(&mesh.strips, &mut strips, base);
        if !mesh.points.is_empty() {
            y_offset += (max_y - min_y).max(0.0) + gap;
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    result
}

/// Tile a mesh in a grid pattern.
pub fn tile_mesh(mesh: &PolyData, nx: usize, ny: usize, spacing: [f64; 2]) -> PolyData {
    let mut all = Vec::new();
    for iy in 0..ny {
        for ix in 0..nx {
            let mut copy = mesh.clone();
            let mut pts = Points::<f64>::new();
            for i in 0..copy.points.len() {
                let p = copy.points.get(i);
                pts.push([
                    p[0] + ix as f64 * spacing[0],
                    p[1] + iy as f64 * spacing[1],
                    p[2],
                ]);
            }
            copy.points = pts;
            all.push(copy);
        }
    }
    let refs: Vec<&PolyData> = all.iter().collect();
    append_meshes(&refs)
}

fn copy_cells(src: &CellArray, dst: &mut CellArray, offset: i64) {
    for cell in src.iter() {
        let ids: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
        dst.push_cell(&ids);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn append() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = append_meshes(&[&a, &b]);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
    }
    #[test]
    fn append_preserves_all_cell_arrays() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([1.0, 1.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);
        a.verts.push_cell(&[0]);
        a.lines.push_cell(&[0, 1]);
        a.polys.push_cell(&[0, 1, 2]);
        a.strips.push_cell(&[0, 1, 2, 3]);

        let result = append_meshes(&[&a, &a]);
        assert_eq!(result.verts.num_cells(), 2);
        assert_eq!(result.lines.cell(1), &[4, 5]);
        assert_eq!(result.polys.cell(1), &[4, 5, 6]);
        assert_eq!(result.strips.cell(1), &[4, 5, 6, 7]);
    }
    #[test]
    fn zip() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = zip_meshes(&a, &b);
        assert_eq!(result.polys.num_cells(), 2);
    }
    #[test]
    fn tile() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = tile_mesh(&mesh, 3, 2, [2.0, 2.0]);
        assert_eq!(result.polys.num_cells(), 6);
    }
    #[test]
    fn stack() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = stack_vertical(&[&a, &b], 0.5);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn stack_normalizes_positive_min_y() {
        let a = PolyData::from_triangles(
            vec![[0.0, 5.0, 0.0], [1.0, 5.0, 0.0], [0.0, 7.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 10.0, 0.0], [1.0, 10.0, 0.0], [0.0, 11.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = stack_vertical(&[&a, &b], 0.5);
        assert_eq!(result.points.get(0)[1], 0.0);
        assert_eq!(result.points.get(3)[1], 2.5);
    }
}
