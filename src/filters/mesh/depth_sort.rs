use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Sort faces by depth (distance from a viewpoint).
///
/// Reorders cells so that faces farther from the viewpoint come first.
/// Useful for transparency rendering (painter's algorithm).
pub fn depth_sort(input: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let mut indexed: Vec<(f64, usize)> = cells
        .iter()
        .enumerate()
        .map(|(fi, c)| {
            if c.is_empty() {
                return (0.0, fi);
            }
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &id in c {
                let p = input.points.get(id as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = c.len() as f64;
            cx /= n;
            cy /= n;
            cz /= n;
            let d2 = (cx - viewpoint[0]).powi(2)
                + (cy - viewpoint[1]).powi(2)
                + (cz - viewpoint[2]).powi(2);
            (d2, fi)
        })
        .collect();

    // Sort far to near (for back-to-front rendering)
    indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let mut out_polys = CellArray::new();
    let mut depth_values = Vec::with_capacity(cells.len());
    for &(d2, fi) in &indexed {
        out_polys.push_cell(&cells[fi]);
        depth_values.push(d2.sqrt());
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Depth",
            depth_values,
            1,
        )));
    pd
}

/// Sort faces front-to-back (for occlusion culling).
pub fn depth_sort_front_to_back(input: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let mut indexed: Vec<(f64, usize)> = cells
        .iter()
        .enumerate()
        .map(|(fi, c)| {
            if c.is_empty() {
                return (0.0, fi);
            }
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &id in c {
                let p = input.points.get(id as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = c.len() as f64;
            cx /= n;
            cy /= n;
            cz /= n;
            (
                (cx - viewpoint[0]).powi(2)
                    + (cy - viewpoint[1]).powi(2)
                    + (cz - viewpoint[2]).powi(2),
                fi,
            )
        })
        .collect();

    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut out_polys = CellArray::new();
    for &(_, fi) in &indexed {
        out_polys.push_cell(&cells[fi]);
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_to_front() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // near
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]); // far
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = depth_sort(&pd, [0.0, 0.0, 0.0]);
        // Far face should come first
        let arr = result.cell_data().get_array("Depth").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let d0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let d1 = buf[0];
        assert!(d0 >= d1); // first cell is farther
    }

    #[test]
    fn front_to_back() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[0, 1, 2]); // far first, near second

        let result = depth_sort_front_to_back(&pd, [0.0, 0.0, 0.0]);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn single_face() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = depth_sort(&pd, [0.0, 0.0, 5.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(depth_sort(&pd, [0.0; 3]).polys.num_cells(), 0);
    }
}
