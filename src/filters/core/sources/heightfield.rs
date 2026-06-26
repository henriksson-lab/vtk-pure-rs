//! Heightfield mesh from a 2D function or data array.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate a heightfield mesh from a function f(x,y) -> z.
pub fn heightfield_from_function(
    x_range: (f64, f64),
    y_range: (f64, f64),
    nx: usize,
    ny: usize,
    f: impl Fn(f64, f64) -> f64,
) -> PolyData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut height = Vec::new();

    for j in 0..=ny {
        for i in 0..=nx {
            let x = x_range.0 + (x_range.1 - x_range.0) * i as f64 / nx as f64;
            let y = y_range.0 + (y_range.1 - y_range.0) * j as f64 / ny as f64;
            let z = f(x, y);
            points.push([x, y, z]);
            height.push(z);
        }
    }

    let row = nx + 1;
    for j in 0..ny {
        for i in 0..nx {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Height", height, 1)));
    mesh.point_data_mut().set_active_scalars("Height");
    mesh
}

/// Generate a heightfield from a flat array of Z values.
pub fn heightfield_from_array(
    x_range: (f64, f64),
    y_range: (f64, f64),
    nx: usize,
    ny: usize,
    z_values: &[f64],
) -> PolyData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let row = nx + 1;

    heightfield_from_function(x_range, y_range, nx, ny, |x, y| {
        let ix = ((x - x_range.0) / (x_range.1 - x_range.0) * nx as f64) as usize;
        let iy = ((y - y_range.0) / (y_range.1 - y_range.0) * ny as f64) as usize;
        let ix = ix.min(nx);
        let iy = iy.min(ny);
        let idx = iy * row + ix;
        if idx < z_values.len() {
            z_values[idx]
        } else {
            0.0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_fn() {
        let hf = heightfield_from_function((-1.0, 1.0), (-1.0, 1.0), 10, 10, |x, y| x * x + y * y);
        assert_eq!(hf.points.len(), 121);
        assert!(hf.point_data().get_array("Height").is_some());
    }
    #[test]
    fn from_array() {
        let z: Vec<f64> = (0..121).map(|i| i as f64 * 0.01).collect();
        let hf = heightfield_from_array((0.0, 1.0), (0.0, 1.0), 10, 10, &z);
        assert_eq!(hf.points.len(), 121);
    }

    #[test]
    fn from_array_clamps_dimensions_consistently() {
        let z = vec![1.0, 2.0, 3.0, 4.0];
        let hf = heightfield_from_array((0.0, 1.0), (0.0, 1.0), 0, 0, &z);
        assert_eq!(hf.points.len(), 4);
        assert_eq!(hf.polys.num_cells(), 2);
        let height = hf
            .point_data()
            .get_array("Height")
            .unwrap()
            .as_f64()
            .unwrap();
        assert_eq!(height.tuple(0), &[1.0]);
        assert_eq!(height.tuple(3), &[4.0]);
    }
}
