use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Generate one point at the center of each non-empty input cell.
pub fn cell_centers(input: &PolyData) -> PolyData {
    let nc = input.total_cells();
    let mut pts = Vec::with_capacity(nc * 3);
    let mut source_cell_ids = Vec::with_capacity(nc);

    let mut cell_id = 0usize;
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            if cell.is_empty() {
                cell_id += 1;
                continue;
            }

            let n = cell.len() as f64;
            let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
            for &id in cell {
                let p = input.points.get(id as usize);
                x += p[0];
                y += p[1];
                z += p[2];
            }

            pts.push(x / n);
            pts.push(y / n);
            pts.push(z / n);
            source_cell_ids.push(cell_id);
            cell_id += 1;
        }
    }

    let mut output = PolyData::new();
    output.points = Points::from_flat_vec(pts);
    if source_cell_ids.len() == input.total_cells() {
        for array in input.cell_data().field_data().iter() {
            output.point_data_mut().add_array(array.clone());
        }
    } else {
        for array in input.cell_data().field_data().iter() {
            let num_comp = array.num_components();
            let mut data = Vec::with_capacity(source_cell_ids.len() * num_comp);
            let mut tuple = vec![0.0; num_comp];
            for &source_cell_id in &source_cell_ids {
                array.tuple_as_f64(source_cell_id, &mut tuple);
                data.extend_from_slice(&tuple);
            }
            output
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    array.name(),
                    data,
                    num_comp,
                )));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_of_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [0.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = cell_centers(&pd);
        assert_eq!(r.points.len(), 2);
        let c = r.points.get(0);
        assert!((c[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn centers_all_polydata_cell_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.strips.push_cell(&[0, 1, 3]);

        let r = cell_centers(&pd);
        assert_eq!(r.points.len(), 4);
        assert_eq!(r.verts.num_cells(), 0);
    }
}
