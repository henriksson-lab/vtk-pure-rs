use crate::data::{CellArray, Points, PolyData};

/// Extract vertices as a point cloud PolyData (strip all cells).
///
/// Creates vertex cells for every point and removes polygon/line/strip cells.
pub fn vertices_only(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut verts = CellArray::new();
    for i in 0..n {
        verts.push_cell(&[i as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.verts = verts;
    // Copy point data
    for i in 0..input.point_data().num_arrays() {
        pd.point_data_mut()
            .add_array(input.point_data().get_array_by_index(i).unwrap().clone());
    }
    pd
}

/// Extract cell centroids as a point cloud.
///
/// Each polygon becomes a single point at its centroid.
/// Cell data becomes point data on the output.
pub fn cell_centroids_as_points(input: &PolyData) -> PolyData {
    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();

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
        let idx = out_pts.len() as i64;
        out_pts.push([cx / n, cy / n, cz / n]);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    // Copy cell data as point data
    for i in 0..input.cell_data().num_arrays() {
        pd.point_data_mut()
            .add_array(input.cell_data().get_array_by_index(i).unwrap().clone());
    }
    pd
}

/// Extract edge midpoints as a point cloud.
pub fn edge_midpoints(input: &PolyData) -> PolyData {
    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut seen = std::collections::HashSet::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = input.points.get(a as usize);
                let pb = input.points.get(b as usize);
                let idx = out_pts.len() as i64;
                out_pts.push([
                    (pa[0] + pb[0]) * 0.5,
                    (pa[1] + pb[1]) * 0.5,
                    (pa[2] + pb[2]) * 0.5,
                ]);
                out_verts.push_cell(&[idx]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn vertices_only_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertices_only(&pd);
        assert_eq!(result.verts.num_cells(), 3);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn cell_centroids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([0.0, 3.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cell_centroids_as_points(&pd);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn edge_midpoints_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = edge_midpoints(&pd);
        assert_eq!(result.points.len(), 3); // 3 edges
    }

    #[test]
    fn preserves_point_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![42.0], 1)));

        let result = vertices_only(&pd);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(vertices_only(&pd).points.len(), 0);
    }
}
