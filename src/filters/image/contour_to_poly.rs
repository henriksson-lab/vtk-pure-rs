//! Extract contour lines from a 2D ImageData scalar field as PolyData.
//!
//! Uses marching squares to produce contour polylines on a 2D grid.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};
use std::collections::HashMap;

/// Extract contour lines at a given isovalue from a 2D ImageData.
///
/// Returns a PolyData with line segments.
pub fn image_contour_to_poly_data(image: &ImageData, array_name: &str, isovalue: f64) -> PolyData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();
    let extent = image.extent();

    if dims[0] < 2 || dims[1] < 2 {
        return PolyData::new();
    }

    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };

    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut point_ids: HashMap<[u64; 3], i64> = HashMap::new();
    let mut contour_scalars = Vec::new();

    // Pre-read all values for closure-safe access
    let n_total = dims[0] * dims[1];
    let mut all_vals = vec![0.0f64; n_total];
    let mut buf = [0.0f64];
    for i in 0..n_total.min(arr.num_tuples()) {
        arr.tuple_as_f64(i, &mut buf);
        all_vals[i] = buf[0];
    }

    let val_at = |ix: usize, iy: usize| -> f64 {
        let idx = ix + iy * dims[0];
        if idx < all_vals.len() {
            all_vals[idx]
        } else {
            0.0
        }
    };

    // Marching squares on each cell
    for iy in 0..dims[1] - 1 {
        for ix in 0..dims[0] - 1 {
            let v00 = val_at(ix, iy);
            let v10 = val_at(ix + 1, iy);
            let v01 = val_at(ix, iy + 1);
            let v11 = val_at(ix + 1, iy + 1);

            // Match vtkMarchingSquares CASE_MASK: { bottom-left, bottom-right,
            // top-left, top-right } => { 1, 2, 8, 4 }.
            let case = ((v00 >= isovalue) as u8)
                | (((v10 >= isovalue) as u8) << 1)
                | (((v01 >= isovalue) as u8) << 3)
                | (((v11 >= isovalue) as u8) << 2);

            if case == 0 || case == 15 {
                continue;
            }

            let x0 = origin[0] + (extent[0] as f64 + ix as f64) * spacing[0];
            let y0 = origin[1] + (extent[2] as f64 + iy as f64) * spacing[1];
            let x1 = x0 + spacing[0];
            let y1 = y0 + spacing[1];
            let z = origin[2] + extent[4] as f64 * spacing[2];

            // Interpolation helpers
            let lerp_x = |va: f64, vb: f64, xa: f64, xb: f64| -> f64 {
                let t = if (vb - va).abs() > 1e-15 {
                    (isovalue - va) / (vb - va)
                } else {
                    0.5
                };
                xa + t * (xb - xa)
            };

            // Edge midpoints
            let bottom = [lerp_x(v00, v10, x0, x1), y0, z];
            let right = [x1, lerp_x(v10, v11, y0, y1), z];
            let top = [lerp_x(v01, v11, x0, x1), y1, z];
            let left = [x0, lerp_x(v00, v01, y0, y1), z];

            let segments: Vec<([f64; 3], [f64; 3])> = match case {
                1 => vec![(bottom, left)],
                2 => vec![(right, bottom)],
                3 => vec![(right, left)],
                4 => vec![(top, right)],
                5 => vec![(bottom, left), (top, right)],
                6 => vec![(top, bottom)],
                7 => vec![(top, left)],
                8 => vec![(left, top)],
                9 => vec![(bottom, top)],
                10 => vec![(right, bottom), (left, top)],
                11 => vec![(right, top)],
                12 => vec![(left, right)],
                13 => vec![(bottom, right)],
                14 => vec![(left, bottom)],
                _ => continue,
            };

            for (p1, p2) in &segments {
                let i1 = insert_unique_point(
                    &mut points,
                    &mut point_ids,
                    &mut contour_scalars,
                    *p1,
                    isovalue,
                );
                let i2 = insert_unique_point(
                    &mut points,
                    &mut point_ids,
                    &mut contour_scalars,
                    *p2,
                    isovalue,
                );
                if i1 != i2 {
                    lines.push_cell(&[i1, i2]);
                }
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            contour_scalars,
            1,
        )));
    mesh
}

fn insert_unique_point(
    points: &mut Points<f64>,
    point_ids: &mut HashMap<[u64; 3], i64>,
    contour_scalars: &mut Vec<f64>,
    point: [f64; 3],
    isovalue: f64,
) -> i64 {
    let key = [point[0].to_bits(), point[1].to_bits(), point[2].to_bits()];
    if let Some(&id) = point_ids.get(&key) {
        return id;
    }

    let id = points.len() as i64;
    points.push(point);
    point_ids.insert(key, id);
    contour_scalars.push(isovalue);
    id
}

/// Extract multiple contour levels at once.
pub fn image_multi_contour(image: &ImageData, array_name: &str, isovalues: &[f64]) -> PolyData {
    if isovalues.is_empty() {
        return PolyData::new();
    }

    let contours: Vec<PolyData> = isovalues
        .iter()
        .map(|&iso| image_contour_to_poly_data(image, array_name, iso))
        .collect();
    // Inline append
    let mut pts = crate::data::Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut scalars = Vec::new();
    for c in &contours {
        let base = pts.len() as i64;
        for i in 0..c.points.len() {
            pts.push(c.points.get(i));
        }
        if let Some(array) = c.point_data().get_array(array_name) {
            scalars.extend(array.to_f64_vec());
        }
        for cell in c.lines.iter() {
            let ids: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            lines.push_cell(&ids);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, scalars, 1,
        )));
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_contour() {
        let image = ImageData::from_function(
            [20, 20, 1],
            [0.1, 0.1, 1.0],
            [0.0, 0.0, 0.0],
            "dist",
            |x, y, _z| (x * x + y * y).sqrt(),
        );
        let contour = image_contour_to_poly_data(&image, "dist", 0.5);
        assert!(contour.lines.num_cells() > 0);
    }

    #[test]
    fn multi_contour() {
        let image = ImageData::from_function(
            [10, 10, 1],
            [0.1, 0.1, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x,
        );
        let contour = image_multi_contour(&image, "val", &[0.3, 0.5, 0.7]);
        assert!(contour.lines.num_cells() > 0);
    }

    #[test]
    fn no_crossing() {
        let image = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |_x, _y, _z| 1.0,
        );
        let contour = image_contour_to_poly_data(&image, "val", 5.0);
        assert_eq!(contour.lines.num_cells(), 0);
    }

    #[test]
    fn coordinates_include_image_extent() {
        let mut image = ImageData::with_dimensions(2, 2, 1);
        image.set_extent([10, 11, 20, 21, 3, 3]);
        image.set_spacing([2.0, 3.0, 4.0]);
        image.set_origin([1.0, 2.0, 5.0]);
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 1.0, 1.0, 1.0],
                1,
            )));

        let contour = image_contour_to_poly_data(&image, "val", 0.5);
        assert_eq!(contour.lines.num_cells(), 1);
        let p0 = contour.points.get(0);
        let p1 = contour.points.get(1);
        assert_eq!(p0, [21.0, 63.5, 17.0]);
        assert_eq!(p1, [22.0, 62.0, 17.0]);
    }
}
