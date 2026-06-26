use crate::data::{AnyDataArray, PolyData};

/// Sample a mesh's scalar field onto a regular 2D grid.
///
/// Projects grid points onto the mesh surface (in XY plane) and interpolates
/// scalar values using barycentric coordinates. Grid covers the mesh's XY
/// bounding box. Returns a `ny x nx` nested Vec of scalar values (NaN where
/// no triangle covers the grid point).
pub fn resample_on_grid(input: &PolyData, array_name: &str, nx: usize, ny: usize) -> Vec<Vec<f64>> {
    if nx == 0 || ny == 0 {
        return vec![];
    }

    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return vec![vec![f64::NAN; nx]; ny],
    };

    let n_pts = input.points.len();
    if n_pts == 0 {
        return vec![vec![f64::NAN; nx]; ny];
    }

    // Compute bounding box in XY
    let mut x_min: f64 = f64::INFINITY;
    let mut x_max: f64 = f64::NEG_INFINITY;
    let mut y_min: f64 = f64::INFINITY;
    let mut y_max: f64 = f64::NEG_INFINITY;

    for i in 0..n_pts {
        let p = input.points.get(i);
        x_min = x_min.min(p[0]);
        x_max = x_max.max(p[0]);
        y_min = y_min.min(p[1]);
        y_max = y_max.max(p[1]);
    }

    let dx: f64 = if nx > 1 {
        (x_max - x_min) / (nx - 1) as f64
    } else {
        1.0
    };
    let dy: f64 = if ny > 1 {
        (y_max - y_min) / (ny - 1) as f64
    } else {
        1.0
    };

    // Read scalar values
    let mut buf = [0.0f64];
    let scalars: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // Collect triangles
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            let tri = [cell[0], cell[1], cell[2]];
            if tri.iter().all(|&id| id >= 0 && (id as usize) < n_pts) {
                tris.push([tri[0] as usize, tri[1] as usize, tri[2] as usize]);
            }
        }
    }

    let mut grid = vec![vec![f64::NAN; nx]; ny];

    for row in 0..ny {
        let gy: f64 = y_min + row as f64 * dy;
        for col in 0..nx {
            let gx: f64 = x_min + col as f64 * dx;

            // Find which triangle contains (gx, gy) in XY projection
            for tri in &tris {
                let p0 = input.points.get(tri[0]);
                let p1 = input.points.get(tri[1]);
                let p2 = input.points.get(tri[2]);

                let (u, v, w) = barycentric_2d(gx, gy, p0, p1, p2);
                if u >= -1e-10 && v >= -1e-10 && w >= -1e-10 {
                    let s0 = if tri[0] < scalars.len() {
                        scalars[tri[0]]
                    } else {
                        0.0
                    };
                    let s1 = if tri[1] < scalars.len() {
                        scalars[tri[1]]
                    } else {
                        0.0
                    };
                    let s2 = if tri[2] < scalars.len() {
                        scalars[tri[2]]
                    } else {
                        0.0
                    };
                    grid[row][col] = u * s0 + v * s1 + w * s2;
                    break;
                }
            }
        }
    }

    grid
}

/// Compute barycentric coordinates of point (px, py) in triangle (p0, p1, p2)
/// projected onto the XY plane.
fn barycentric_2d(px: f64, py: f64, p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> (f64, f64, f64) {
    let v0x: f64 = p1[0] - p0[0];
    let v0y: f64 = p1[1] - p0[1];
    let v1x: f64 = p2[0] - p0[0];
    let v1y: f64 = p2[1] - p0[1];
    let v2x: f64 = px - p0[0];
    let v2y: f64 = py - p0[1];

    let d00: f64 = v0x * v0x + v0y * v0y;
    let d01: f64 = v0x * v1x + v0y * v1y;
    let d11: f64 = v1x * v1x + v1y * v1y;
    let d20: f64 = v2x * v0x + v2y * v0y;
    let d21: f64 = v2x * v1x + v2y * v1y;

    let denom: f64 = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-30 {
        return (-1.0, -1.0, -1.0);
    }

    let v: f64 = (d11 * d20 - d01 * d21) / denom;
    let w: f64 = (d00 * d21 - d01 * d20) / denom;
    let u: f64 = 1.0 - v - w;
    (u, v, w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, PolyData};

    fn make_triangle_with_scalar() -> PolyData {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![0.0, 1.0, 2.0],
                1,
            )));
        pd
    }

    #[test]
    fn grid_size_matches() {
        let pd = make_triangle_with_scalar();
        let grid = resample_on_grid(&pd, "temperature", 5, 4);
        assert_eq!(grid.len(), 4);
        assert_eq!(grid[0].len(), 5);
    }

    #[test]
    fn corner_values_match() {
        let pd = make_triangle_with_scalar();
        let grid = resample_on_grid(&pd, "temperature", 3, 3);
        // Bottom-left corner (0,0) should be scalar 0.0 (vertex 0)
        assert!((grid[0][0] - 0.0).abs() < 1e-6);
        // Bottom-right corner (2,0) should be scalar 1.0 (vertex 1)
        assert!((grid[0][2] - 1.0).abs() < 1e-6);
        // Top-left corner (0,2) should be scalar 2.0 (vertex 2)
        assert!((grid[2][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn missing_array_returns_nan() {
        let pd = make_triangle_with_scalar();
        let grid = resample_on_grid(&pd, "nonexistent", 3, 3);
        assert!(grid[0][0].is_nan());
    }
}
