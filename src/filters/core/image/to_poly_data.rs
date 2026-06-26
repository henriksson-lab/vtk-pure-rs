use crate::data::{ImageData, Points, PolyData};

/// Convert an ImageData extent to PolyData geometry.
///
/// Mirrors the default behavior of VTK's `vtkImageDataGeometryFilter`: a
/// degenerate extent becomes vertices, lines, or quads, while a full 3-D extent
/// is emitted as one vertex cell per image point.
pub fn image_data_to_surface(image: &ImageData) -> PolyData {
    let dims = image.dimensions();
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 {
        return PolyData::new();
    }

    let diff = [dims[0] - 1, dims[1] - 1, dims[2] - 1];
    let dimension = diff.iter().filter(|&&d| d > 0).count();
    let mut points = Points::new();

    let mut pd = PolyData::new();
    match dimension {
        0 => {
            points.push(image.point_from_ijk(0, 0, 0));
            pd.verts.push_cell(&[0]);
            copy_point_data(image, &[0], &mut pd);
        }
        1 => {
            let dir = diff.iter().position(|&d| d > 0).unwrap();
            for i in 0..=diff[dir] {
                let ijk = ijk_on_axis(dir, i);
                points.push(image.point_from_ijk(ijk[0], ijk[1], ijk[2]));
            }
            for i in 0..diff[dir] {
                pd.lines.push_cell(&[i as i64, i as i64 + 1]);
            }
            let ids: Vec<usize> = (0..=diff[dir])
                .map(|i| flat_index(dims, ijk_on_axis(dir, i)))
                .collect();
            copy_point_data(image, &ids, &mut pd);
        }
        2 => {
            let mut dirs = [0usize; 2];
            let mut idx = 0;
            for (axis, &d) in diff.iter().enumerate() {
                if d > 0 {
                    dirs[idx] = axis;
                    idx += 1;
                }
            }
            let n0 = diff[dirs[0]] + 1;
            let n1 = diff[dirs[1]] + 1;
            let mut ids = Vec::with_capacity(n0 * n1);
            for j in 0..n1 {
                for i in 0..n0 {
                    let ijk = ijk_on_axes(dirs[0], dirs[1], i, j);
                    points.push(image.point_from_ijk(ijk[0], ijk[1], ijk[2]));
                    ids.push(flat_index(dims, ijk));
                }
            }
            for j in 0..diff[dirs[1]] {
                for i in 0..diff[dirs[0]] {
                    let p0 = (i + j * n0) as i64;
                    let p1 = p0 + 1;
                    let p2 = p1 + n0 as i64;
                    let p3 = p2 - 1;
                    pd.polys.push_cell(&[p0, p1, p2, p3]);
                }
            }
            copy_point_data(image, &ids, &mut pd);
        }
        _ => {
            let n = dims[0] * dims[1] * dims[2];
            let mut ids = Vec::with_capacity(n);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        points.push(image.point_from_ijk(i, j, k));
                        let out_id = ids.len() as i64;
                        pd.verts.push_cell(&[out_id]);
                        ids.push(flat_index(dims, [i, j, k]));
                    }
                }
            }
            copy_point_data(image, &ids, &mut pd);
        }
    }

    pd.points = points;
    pd
}

fn ijk_on_axis(axis: usize, value: usize) -> [usize; 3] {
    let mut ijk = [0usize; 3];
    ijk[axis] = value;
    ijk
}

fn ijk_on_axes(axis0: usize, axis1: usize, a: usize, b: usize) -> [usize; 3] {
    let mut ijk = [0usize; 3];
    ijk[axis0] = a;
    ijk[axis1] = b;
    ijk
}

fn flat_index(dims: [usize; 3], ijk: [usize; 3]) -> usize {
    ijk[0] + ijk[1] * dims[0] + ijk[2] * dims[0] * dims[1]
}

fn copy_point_data(image: &ImageData, input_ids: &[usize], output: &mut PolyData) {
    for array_index in 0..image.point_data().num_arrays() {
        let Some(array) = image.point_data().get_array_by_index(array_index) else {
            continue;
        };
        let components = array.num_components();
        let mut values = Vec::with_capacity(input_ids.len() * components);
        let mut tuple = vec![0.0; components];
        for &input_id in input_ids {
            array.tuple_as_f64(input_id, &mut tuple);
            values.extend_from_slice(&tuple);
        }
        output
            .point_data_mut()
            .add_array(crate::data::AnyDataArray::F64(
                crate::data::DataArray::from_vec(array.name(), values, components),
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_image_surface() {
        let image = ImageData::with_dimensions(3, 3, 3);
        let pd = image_data_to_surface(&image);
        assert_eq!(pd.verts.num_cells(), 27);
        assert_eq!(pd.points.len(), 27);
    }

    #[test]
    fn minimal_image() {
        let image = ImageData::with_dimensions(2, 2, 2);
        let pd = image_data_to_surface(&image);
        assert_eq!(pd.verts.num_cells(), 8);
    }

    #[test]
    fn too_small() {
        let image = ImageData::with_dimensions(1, 2, 2);
        let pd = image_data_to_surface(&image);
        assert_eq!(pd.polys.num_cells(), 1);
    }
}
