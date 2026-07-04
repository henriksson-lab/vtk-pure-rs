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
    let mut out_polys = CellArray::new();
    let mut point_arrays = collect_point_arrays(input);
    let mut cell_arrays = collect_cell_arrays(input);

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 || n_subdivisions == 0 {
            out_polys.push_cell(cell);
            copy_cell_arrays(&mut cell_arrays, cell_id);
            continue;
        }

        let mut polygons = vec![cell.to_vec()];
        for _ in 0..n_subdivisions {
            polygons = subdivide_polygons(polygons, &mut out_points, &mut point_arrays);
        }
        for polygon in polygons {
            out_polys.push_cell(&polygon);
            copy_cell_arrays(&mut cell_arrays, cell_id);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    add_point_arrays(&mut pd, point_arrays);
    add_cell_arrays(&mut pd, cell_arrays);
    pd
}

fn subdivide_polygons(
    polygons: Vec<Vec<i64>>,
    points: &mut crate::data::Points<f64>,
    point_arrays: &mut [PointArray],
) -> Vec<Vec<i64>> {
    let mut new_polygons = Vec::new();
    for polygon in polygons {
        if polygon.len() < 3 {
            new_polygons.push(polygon);
            continue;
        }

        let centroid_id = insert_centroid(&polygon, points, point_arrays);
        for i in 0..polygon.len() {
            new_polygons.push(vec![
                polygon[i],
                polygon[(i + 1) % polygon.len()],
                centroid_id,
            ]);
        }
    }
    new_polygons
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
