use crate::data::{CellArray, DataSet, Points, PolyData};
use std::collections::HashMap;

/// Select faces whose centroid is inside a bounding box.
pub fn select_faces_in_box(input: &PolyData, bounds: [f64; 6]) -> PolyData {
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;

        if cx >= bounds[0]
            && cx <= bounds[1]
            && cy >= bounds[2]
            && cy <= bounds[3]
            && cz >= bounds[4]
            && cz <= bounds[5]
        {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

/// Select faces whose centroid is inside a sphere.
pub fn select_faces_in_sphere(input: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let r2 = radius * radius;
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;

        let d2 = (cx - center[0]).powi(2) + (cy - center[1]).powi(2) + (cz - center[2]).powi(2);
        if d2 <= r2 {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_in_box() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // centroid ~(0.5,0.33,0)
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]); // centroid ~(10.5,0.33,0)
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = select_faces_in_box(&pd, [0.0, 2.0, 0.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1); // only first face
    }

    #[test]
    fn select_in_sphere() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = select_faces_in_sphere(&pd, [0.5, 0.5, 0.0], 2.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn select_none() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = select_faces_in_box(&pd, [100.0, 200.0, 100.0, 200.0, 100.0, 200.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(select_faces_in_box(&pd, [0.0; 6]).polys.num_cells(), 0);
    }
}
