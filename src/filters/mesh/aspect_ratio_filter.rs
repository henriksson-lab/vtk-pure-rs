use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Remove triangles with aspect ratio exceeding a threshold.
///
/// Aspect ratio = longest_edge / shortest_edge. Removes slivers.
pub fn remove_high_aspect_ratio(input: &PolyData, max_ratio: f64) -> PolyData {
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        let d01 = dist(v0, v1);
        let d12 = dist(v1, v2);
        let d20 = dist(v2, v0);
        let longest = d01.max(d12).max(d20);
        let shortest = d01.min(d12).min(d20);
        let ratio = if shortest > 1e-15 {
            longest / shortest
        } else {
            f64::MAX
        };

        if ratio <= max_ratio {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Extract only triangles with aspect ratio above threshold (select slivers).
pub fn extract_high_aspect_ratio(input: &PolyData, min_ratio: f64) -> PolyData {
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let d01 = dist(v0, v1);
        let d12 = dist(v1, v2);
        let d20 = dist(v2, v0);
        let longest = d01.max(d12).max(d20);
        let shortest = d01.min(d12).min(d20);
        let ratio = if shortest > 1e-15 {
            longest / shortest
        } else {
            f64::MAX
        };

        if ratio >= min_ratio {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_sliver() {
        let mut pd = PolyData::new();
        // Good triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        // Sliver (very thin): long=10, short=0.001
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([10.0, 2.0, 0.0]);
        pd.points.push([10.0, 2.001, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = remove_high_aspect_ratio(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 1); // sliver removed
    }

    #[test]
    fn keep_equilateral() {
        let mut pd = PolyData::new();
        let h = (3.0f64).sqrt() / 2.0;
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_high_aspect_ratio(&pd, 2.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn extract_slivers() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([10.0, 2.0, 0.0]);
        pd.points.push([10.0, 2.001, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = extract_high_aspect_ratio(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(remove_high_aspect_ratio(&pd, 5.0).polys.num_cells(), 0);
    }
}
