use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Densify polygon cells by recursively fanning triangles from each centroid.
///
/// Mirrors VTK's `vtkDensifyPolyData`: triangles, quads, and polygons are
/// replaced by triangles from their centroid; other cells are passed through.
/// The second argument is interpreted as VTK's `NumberOfSubdivisions`.
pub fn densify(input: &PolyData, number_of_subdivisions: f64) -> PolyData {
    let n_subdivisions = if number_of_subdivisions <= 0.0 {
        0
    } else {
        number_of_subdivisions.ceil() as usize
    };
    let mut out_points = input.points.clone();
    let (output_cells, output_connectivity) = estimate_output_polys(&input.polys, n_subdivisions);
    let mut out_offsets = Vec::with_capacity(output_cells + 1);
    let mut out_connectivity = Vec::with_capacity(output_connectivity);
    out_offsets.push(0);
    let mut point_arrays = collect_point_arrays(input);
    let mut cell_arrays = collect_cell_arrays(input);
    reserve_cell_arrays(&mut cell_arrays, output_cells);

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 || n_subdivisions == 0 {
            push_output_cell(cell, &mut out_offsets, &mut out_connectivity);
            copy_cell_arrays(&mut cell_arrays, cell_id);
            continue;
        }

        subdivide_cell(
            cell,
            n_subdivisions,
            &mut out_points,
            &mut point_arrays,
            &mut out_offsets,
            &mut out_connectivity,
        );
        for _ in 0..subdivided_cell_count(cell.len(), n_subdivisions) {
            copy_cell_arrays(&mut cell_arrays, cell_id);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = CellArray::from_raw(out_offsets, out_connectivity);
    add_point_arrays(&mut pd, point_arrays);
    add_cell_arrays(&mut pd, cell_arrays);
    pd
}

fn estimate_output_polys(cells: &CellArray, n_subdivisions: usize) -> (usize, usize) {
    let mut num_cells = 0;
    let mut connectivity_len = 0;
    for cell in cells.iter() {
        if cell.len() < 3 || n_subdivisions == 0 {
            num_cells += 1;
            connectivity_len += cell.len();
        } else {
            let generated = subdivided_cell_count(cell.len(), n_subdivisions);
            num_cells += generated;
            connectivity_len += generated * 3;
        }
    }
    (num_cells, connectivity_len)
}

fn subdivided_cell_count(num_vertices: usize, n_subdivisions: usize) -> usize {
    if n_subdivisions == 0 {
        1
    } else {
        num_vertices * 3usize.pow((n_subdivisions - 1) as u32)
    }
}

fn reserve_cell_arrays(arrays: &mut [CellArrayData], output_cells: usize) {
    for array in arrays {
        array
            .output_data
            .reserve(output_cells * array.num_components);
    }
}

fn push_output_cell(cell: &[i64], offsets: &mut Vec<i64>, connectivity: &mut Vec<i64>) {
    connectivity.extend_from_slice(cell);
    offsets.push(connectivity.len() as i64);
}

fn subdivide_cell(
    polygon: &[i64],
    subdivisions_left: usize,
    points: &mut crate::data::Points<f64>,
    point_arrays: &mut [PointArray],
    offsets: &mut Vec<i64>,
    connectivity: &mut Vec<i64>,
) {
    if polygon.len() < 3 || subdivisions_left == 0 {
        push_output_cell(polygon, offsets, connectivity);
        return;
    }

    let centroid_id = insert_centroid(polygon, points, point_arrays);
    for i in 0..polygon.len() {
        let triangle = [polygon[i], polygon[(i + 1) % polygon.len()], centroid_id];
        subdivide_cell(
            &triangle,
            subdivisions_left - 1,
            points,
            point_arrays,
            offsets,
            connectivity,
        );
    }
}

fn insert_centroid(
    polygon: &[i64],
    points: &mut crate::data::Points<f64>,
    point_arrays: &mut [PointArray],
) -> i64 {
    let mut centroid = [0.0; 3];
    for &id in polygon {
        let p = points.get(id as usize);
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }
    let n = polygon.len() as f64;
    centroid[0] /= n;
    centroid[1] /= n;
    centroid[2] /= n;

    let id = points.len() as i64;
    points.push(centroid);
    for array in point_arrays {
        array.push_average(polygon);
    }
    id
}

struct PointArray {
    name: String,
    num_components: usize,
    data: Vec<f64>,
}

struct CellArrayData {
    name: String,
    num_components: usize,
    input_data: Vec<f64>,
    output_data: Vec<f64>,
}

impl PointArray {
    fn push_average(&mut self, ids: &[i64]) {
        for c in 0..self.num_components {
            let mut sum = 0.0;
            for &id in ids {
                sum += self.data[id as usize * self.num_components + c];
            }
            self.data.push(sum / ids.len() as f64);
        }
    }
}

impl CellArrayData {
    fn copy_tuple(&mut self, cell_id: usize) {
        let start = cell_id * self.num_components;
        self.output_data
            .extend_from_slice(&self.input_data[start..start + self.num_components]);
    }
}

fn collect_point_arrays(input: &PolyData) -> Vec<PointArray> {
    input
        .point_data()
        .iter()
        .map(|array| PointArray {
            name: array.name().to_string(),
            num_components: array.num_components(),
            data: array.to_f64_vec_flat(),
        })
        .collect()
}

fn collect_cell_arrays(input: &PolyData) -> Vec<CellArrayData> {
    input
        .cell_data()
        .iter()
        .map(|array| CellArrayData {
            name: array.name().to_string(),
            num_components: array.num_components(),
            input_data: array.to_f64_vec_flat(),
            output_data: Vec::new(),
        })
        .collect()
}

fn copy_cell_arrays(arrays: &mut [CellArrayData], cell_id: usize) {
    for array in arrays {
        array.copy_tuple(cell_id);
    }
}

fn add_point_arrays(output: &mut PolyData, arrays: Vec<PointArray>) {
    for array in arrays {
        output
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &array.name,
                array.data,
                array.num_components,
            )));
    }
}

fn add_cell_arrays(output: &mut PolyData, arrays: Vec<CellArrayData>) {
    for array in arrays {
        output
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &array.name,
                array.output_data,
                array.num_components,
            )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_subdivisions_passes_cell() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = densify(&pd, 0.0);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn one_subdivision_fans_from_centroid() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = densify(&pd, 1.0);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 3);
        assert!(result.polys.iter().all(|cell| cell.len() == 3));
    }

    #[test]
    fn subdivide_quad_to_four_triangles() {
        let pd = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let result = densify(&pd, 1.0);
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.polys.num_cells(), 4);
    }
}
