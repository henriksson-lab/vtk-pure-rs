use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Clip PolyData by an axis-aligned bounding box.
///
/// Removes cells whose centroid falls outside the specified bounds.
/// `bounds` is `[xmin, xmax, ymin, ymax, zmin, zmax]`.
pub fn box_clip(input: &PolyData, bounds: [f64; 6]) -> PolyData {
    let [xmin, xmax, ymin, ymax, zmin, zmax] = bounds;

    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    let n_cells = input.polys.num_cells();
    for ci in 0..n_cells {
        let cell = input.polys.cell(ci);
        if cell.is_empty() {
            continue;
        }

        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &pid in cell {
            let p = input.points.get(pid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;

        // Test centroid against bounds
        if cx < xmin || cx > xmax || cy < ymin || cy > ymax || cz < zmin || cz > zmax {
            continue;
        }

        // Remap point indices
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| {
                let uid = id as usize;
                *point_map.entry(uid).or_insert_with(|| {
                    let idx = out_points.len();
                    out_points.push(input.points.get(uid));
                    idx
                }) as i64
            })
            .collect();

        out_polys.push_cell(&remapped);
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.polys = out_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_keeps_inside() {
        let pd = PolyData::from_triangles(
            vec![
                [0.5, 0.5, 0.0],
                [1.0, 0.5, 0.0],
                [0.75, 1.0, 0.0],
                // Outside triangle
                [5.0, 5.0, 0.0],
                [6.0, 5.0, 0.0],
                [5.5, 6.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let result = box_clip(&pd, [0.0, 2.0, 0.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn clip_removes_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [10.0, 20.0, 10.0, 20.0, 10.0, 20.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [-10.0, 10.0, -10.0, 10.0, -10.0, 10.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
