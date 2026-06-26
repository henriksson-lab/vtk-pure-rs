//! Extract contour lines from a 2D ImageData scalar field as PolyData.
//!
//! Uses marching squares to produce contour polylines on a 2D grid.

use crate::data::{CellArray, ImageData, Points, PolyData};

/// Extract contour lines at a given isovalue from a 2D ImageData.
///
/// Returns a PolyData with line segments.
pub fn image_contour_to_poly_data(image: &ImageData, array_name: &str, isovalue: f64) -> PolyData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();

    if dims[0] < 2 || dims[1] < 2 {
        return PolyData::new();
    }

    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };

    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();

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
            let v11 = val_at(ix + 1, iy + 1);
            let v01 = val_at(ix, iy + 1);

            let case = ((v00 >= isovalue) as u8)
                | (((v10 >= isovalue) as u8) << 1)
                | (((v11 >= isovalue) as u8) << 2)
                | (((v01 >= isovalue) as u8) << 3);

            if case == 0 || case == 15 {
                continue;
            }

            let x0 = origin[0] + ix as f64 * spacing[0];
            let y0 = origin[1] + iy as f64 * spacing[1];
            let x1 = x0 + spacing[0];
            let y1 = y0 + spacing[1];

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
            let z = origin[2];
            let bottom = [lerp_x(v00, v10, x0, x1), y0, z];
            let right = [x1, lerp_x(v10, v11, y0, y1), z];
            let top = [lerp_x(v01, v11, x0, x1), y1, z];
            let left = [x0, lerp_x(v00, v01, y0, y1), z];

            let segments: Vec<([f64; 3], [f64; 3])> = match case {
                1 | 14 => vec![(bottom, left)],
                2 | 13 => vec![(bottom, right)],
                3 | 12 => vec![(left, right)],
                4 | 11 => vec![(right, top)],
                5 => vec![(bottom, right), (left, top)],
                6 | 9 => vec![(bottom, top)],
                7 | 8 => vec![(left, top)],
                10 => vec![(bottom, left), (right, top)],
                _ => continue,
            };

            for (p1, p2) in &segments {
                let i1 = points.len() as i64;
                points.push(*p1);
                let i2 = points.len() as i64;
                points.push(*p2);
                lines.push_cell(&[i1, i2]);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh
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
    for c in &contours {
        let base = pts.len() as i64;
        for i in 0..c.points.len() {
            pts.push(c.points.get(i));
        }
        for cell in c.lines.iter() {
            let ids: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            lines.push_cell(&ids);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
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
    fn preserves_origin_z() {
        let image = ImageData::from_function(
            [2, 2, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 7.0],
            "val",
            |x, _y, _z| x,
        );
        let contour = image_contour_to_poly_data(&image, "val", 0.5);
        assert!(contour.points.len() > 0);
        assert_eq!(contour.points.get(0)[2], 7.0);
    }
}
