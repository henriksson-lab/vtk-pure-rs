use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a cube (axis-aligned box).
pub struct CubeParams {
    pub center: [f64; 3],
    pub x_length: f64,
    pub y_length: f64,
    pub z_length: f64,
}

impl Default for CubeParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            x_length: 1.0,
            y_length: 1.0,
            z_length: 1.0,
        }
    }
}

/// Generate a cube as PolyData with 24 face vertices, 6 quad faces, and normals.
pub fn cube(params: &CubeParams) -> PolyData {
    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TCoords", 2);
    let mut polys = CellArray::new();

    let x_length = params.x_length.abs();
    let y_length = params.y_length.abs();
    let z_length = params.z_length.abs();

    let mut x = [0.0; 3];
    let mut n = [0.0; 3];
    let mut tc = [0.0; 2];

    x[0] = params.center[0] - x_length / 2.0;
    n[0] = -1.0;
    n[1] = 0.0;
    n[2] = 0.0;
    for i in 0..2 {
        x[1] = params.center[1] - y_length / 2.0;
        for _j in 0..2 {
            tc[1] = x[1] + 0.5;
            x[2] = params.center[2] - z_length / 2.0;
            for _k in 0..2 {
                tc[0] = (x[2] + 0.5) * (1.0 - 2.0 * i as f64);
                points.push(x);
                tcoords.push_tuple(&tc);
                normals.push_tuple(&n);
                x[2] += z_length;
            }
            x[1] += y_length;
        }
        x[0] += x_length;
        n[0] += 2.0;
    }
    polys.push_cell(&[0, 1, 3, 2]);
    polys.push_cell(&[4, 6, 7, 5]);

    x[1] = params.center[1] - y_length / 2.0;
    n[0] = 0.0;
    n[1] = -1.0;
    n[2] = 0.0;
    for i in 0..2 {
        x[0] = params.center[0] - x_length / 2.0;
        for _j in 0..2 {
            tc[0] = (x[0] + 0.5) * (2.0 * i as f64 - 1.0);
            x[2] = params.center[2] - z_length / 2.0;
            for _k in 0..2 {
                tc[1] = (x[2] + 0.5) * -1.0;
                points.push(x);
                tcoords.push_tuple(&tc);
                normals.push_tuple(&n);
                x[2] += z_length;
            }
            x[0] += x_length;
        }
        x[1] += y_length;
        n[1] += 2.0;
    }
    polys.push_cell(&[8, 10, 11, 9]);
    polys.push_cell(&[12, 13, 15, 14]);

    x[2] = params.center[2] - z_length / 2.0;
    n[0] = 0.0;
    n[1] = 0.0;
    n[2] = -1.0;
    for i in 0..2 {
        x[1] = params.center[1] - y_length / 2.0;
        for _j in 0..2 {
            tc[1] = x[1] + 0.5;
            x[0] = params.center[0] - x_length / 2.0;
            for _k in 0..2 {
                tc[0] = (x[0] + 0.5) * (2.0 * i as f64 - 1.0);
                points.push(x);
                tcoords.push_tuple(&tc);
                normals.push_tuple(&n);
                x[0] += x_length;
            }
            x[1] += y_length;
        }
        x[2] += z_length;
        n[2] += 2.0;
    }
    polys.push_cell(&[16, 18, 19, 17]);
    polys.push_cell(&[20, 21, 23, 22]);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cube() {
        let pd = cube(&CubeParams::default());
        assert_eq!(pd.points.len(), 24); // 4 vertices per face * 6 faces
        assert_eq!(pd.polys.num_cells(), 6);
        assert_eq!(pd.polys.cell(0), &[0, 1, 3, 2]);
        assert_eq!(pd.polys.cell(5), &[20, 21, 23, 22]);
        assert!(pd.point_data().normals().is_some());
        assert!(pd.point_data().tcoords().is_some());
    }

    #[test]
    fn negative_lengths_are_nonnegative_like_vtk() {
        let pd = cube(&CubeParams {
            center: [0.0, 0.0, 0.0],
            x_length: -2.0,
            y_length: -4.0,
            z_length: -6.0,
        });

        let bounds = pd.points.bounds();
        assert_eq!(bounds.x_min, -1.0);
        assert_eq!(bounds.x_max, 1.0);
        assert_eq!(bounds.y_min, -2.0);
        assert_eq!(bounds.y_max, 2.0);
        assert_eq!(bounds.z_min, -3.0);
        assert_eq!(bounds.z_max, 3.0);
    }
}
