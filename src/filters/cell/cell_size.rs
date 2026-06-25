use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute VTK-style size arrays for each cell.
///
/// Adds "VertexCount", "Length", "Area", and "Volume" arrays to cell data.
pub fn cell_size(input: &PolyData) -> PolyData {
    let num_cells = input.total_cells();
    let mut vertex_counts = vec![0.0; num_cells];
    let mut lengths = vec![0.0; num_cells];
    let mut areas = vec![0.0; num_cells];
    let volumes = vec![0.0; num_cells];
    let mut cell_sizes = vec![0.0; num_cells];

    let mut cell_id = 0usize;
    for cell in input.verts.iter() {
        vertex_counts[cell_id] = cell.len() as f64;
        cell_sizes[cell_id] = vertex_counts[cell_id];
        cell_id += 1;
    }
    for cell in input.lines.iter() {
        lengths[cell_id] = polyline_length(input, cell);
        cell_sizes[cell_id] = lengths[cell_id];
        cell_id += 1;
    }
    for cell in input.polys.iter() {
        areas[cell_id] = polygon_area(input, cell);
        cell_sizes[cell_id] = areas[cell_id];
        cell_id += 1;
    }
    for cell in input.strips.iter() {
        areas[cell_id] = triangle_strip_area(input, cell);
        cell_sizes[cell_id] = areas[cell_id];
        cell_id += 1;
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexCount",
            vertex_counts,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Length", lengths, 1)));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Volume", volumes, 1)));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellSize", cell_sizes, 1,
        )));
    pd
}

/// Compute the edge length of each line cell.
pub fn cell_size_lines(input: &PolyData) -> PolyData {
    let mut lengths = Vec::new();

    for cell in input.lines.iter() {
        lengths.push(polyline_length(input, cell));
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellSize", lengths, 1,
        )));
    pd
}

fn polygon_area(input: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }

    let p0 = input.points.get(cell[0] as usize);
    let mut total_area = 0.0;

    for i in 1..cell.len() - 1 {
        let p1 = input.points.get(cell[i] as usize);
        let p2 = input.points.get(cell[i + 1] as usize);
        total_area += triangle_area(p0, p1, p2);
    }

    total_area
}

fn triangle_strip_area(input: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }

    let mut total_area = 0.0;
    for i in 0..cell.len() - 2 {
        let p0 = input.points.get(cell[i] as usize);
        let p1 = input.points.get(cell[i + 1] as usize);
        let p2 = input.points.get(cell[i + 2] as usize);
        total_area += triangle_area(p0, p1, p2);
    }
    total_area
}

fn triangle_area(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> f64 {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

fn polyline_length(input: &PolyData, cell: &[i64]) -> f64 {
    let mut total_len = 0.0;
    for i in 0..cell.len().saturating_sub(1) {
        let p0 = input.points.get(cell[i] as usize);
        let p1 = input.points.get(cell[i + 1] as usize);
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        total_len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    total_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_area() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = cell_size(&pd);
        let arr = result.cell_data().get_array("Area").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn quad_area() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 3.0, 0.0]);
        pd.points.push([0.0, 3.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        let result = cell_size(&pd);
        let arr = result.cell_data().get_array("Area").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn line_length() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        let result = cell_size_lines(&pd);
        let arr = result.cell_data().get_array("CellSize").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn vtk_named_arrays_cover_all_cell_kinds() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.points.push([0.0, 4.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cell_size(&pd);
        assert!(result.cell_data().get_array("VertexCount").is_some());
        assert!(result.cell_data().get_array("Length").is_some());
        assert!(result.cell_data().get_array("Area").is_some());
        assert!(result.cell_data().get_array("Volume").is_some());
        assert!(result.cell_data().get_array("CellSize").is_some());
    }
}
