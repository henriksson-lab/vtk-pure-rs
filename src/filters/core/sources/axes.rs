use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate XYZ axis lines (a triad) for orientation visualization.
///
/// Creates the same point, line, scalar, and normal layout as VTK's `vtkAxes`
/// with origin `(0, 0, 0)`, the requested scale factor, no symmetry, and normals.
pub fn axes(length: f64) -> PolyData {
    axes_with_options([0.0, 0.0, 0.0], length, false, true)
}

/// Generate VTK-style axes with explicit origin, symmetry, and normal controls.
pub fn axes_with_options(
    origin: [f64; 3],
    scale_factor: f64,
    symmetric: bool,
    compute_normals: bool,
) -> PolyData {
    let mut points = Points::new();
    let mut lines = CellArray::new();
    let mut scalars = Vec::with_capacity(6);
    let mut normals = Vec::with_capacity(18);

    let x0 = if symmetric {
        [origin[0] - scale_factor, origin[1], origin[2]]
    } else {
        origin
    };
    points.push(x0);
    scalars.push(0.0);
    normals.extend_from_slice(&[0.0, 1.0, 0.0]);
    points.push([origin[0] + scale_factor, origin[1], origin[2]]);
    scalars.push(0.0);
    normals.extend_from_slice(&[0.0, 1.0, 0.0]);
    lines.push_cell(&[0, 1]);

    let y0 = if symmetric {
        [origin[0], origin[1] - scale_factor, origin[2]]
    } else {
        origin
    };
    points.push(y0);
    scalars.push(0.25);
    normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    points.push([origin[0], origin[1] + scale_factor, origin[2]]);
    scalars.push(0.25);
    normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    lines.push_cell(&[2, 3]);

    let z0 = if symmetric {
        [origin[0], origin[1], origin[2] - scale_factor]
    } else {
        origin
    };
    points.push(z0);
    scalars.push(0.5);
    normals.extend_from_slice(&[1.0, 0.0, 0.0]);
    points.push([origin[0], origin[1], origin[2] + scale_factor]);
    scalars.push(0.5);
    normals.extend_from_slice(&[1.0, 0.0, 0.0]);
    lines.push_cell(&[4, 5]);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Axes", scalars, 1)));
    pd.point_data_mut().set_active_scalars("Axes");
    if compute_normals {
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals", normals, 3,
            )));
        pd.point_data_mut().set_active_normals("Normals");
    }
    pd
}

/// Generate labeled XYZ axes with arrowhead cones.
pub fn axes_with_labels(length: f64, tip_length: f64, tip_radius: f64) -> PolyData {
    let base = axes(length);

    // Generate arrow tips
    let cone_res = 8;
    let mut all_parts = vec![base];

    let directions = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let cone_colors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for (dir, color) in directions.iter().zip(cone_colors.iter()) {
        let tip = make_cone_tip(
            [dir[0] * length, dir[1] * length, dir[2] * length],
            *dir,
            tip_length,
            tip_radius,
            cone_res,
            *color,
        );
        all_parts.push(tip);
    }

    let refs: Vec<&PolyData> = all_parts.iter().collect();
    crate::filters::core::append::append(&refs)
}

fn make_cone_tip(
    base_center: [f64; 3],
    direction: [f64; 3],
    height: f64,
    radius: f64,
    resolution: usize,
    color: [f64; 3],
) -> PolyData {
    let tip = [
        base_center[0] + direction[0] * height,
        base_center[1] + direction[1] * height,
        base_center[2] + direction[2] * height,
    ];

    // Perpendicular frame
    let seed = if direction[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = normalize(cross(seed, direction));
    let v = cross(direction, u);

    let mut points = Points::new();
    let mut polys = CellArray::new();
    let mut colors = DataArray::<f64>::new("Colors", 3);

    // Tip point
    points.push(tip);
    colors.push_tuple(&color);

    // Base ring
    for i in 0..resolution {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / resolution as f64;
        let ct = angle.cos();
        let st = angle.sin();
        points.push([
            base_center[0] + radius * (ct * u[0] + st * v[0]),
            base_center[1] + radius * (ct * u[1] + st * v[1]),
            base_center[2] + radius * (ct * u[2] + st * v[2]),
        ]);
        colors.push_tuple(&color);
    }

    // Side triangles
    for i in 0..resolution {
        let next = (i + 1) % resolution;
        polys.push_cell(&[0, (1 + next) as i64, (1 + i) as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(colors.into());
    pd.point_data_mut().set_active_scalars("Colors");
    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-10 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_axes() {
        let pd = axes(1.0);
        assert_eq!(pd.points.len(), 6);
        assert_eq!(pd.lines.num_cells(), 3);
        assert!(pd.point_data().get_array("Axes").is_some());
        assert!(pd.point_data().get_array("Normals").is_some());
        // X axis endpoint
        let p = pd.points.get(1);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn symmetric_axes() {
        let pd = axes_with_options([1.0, 2.0, 3.0], 0.5, true, false);
        assert_eq!(pd.points.get(0), [0.5, 2.0, 3.0]);
        assert_eq!(pd.points.get(1), [1.5, 2.0, 3.0]);
        assert!(pd.point_data().get_array("Normals").is_none());
    }

    #[test]
    fn axes_with_cones() {
        let pd = axes_with_labels(1.0, 0.2, 0.05);
        // 3 line endpoints (6 pts) + 3 cones (each 1+8=9 pts) = 33
        assert!(pd.points.len() > 6);
        assert!(pd.polys.num_cells() > 0); // cone triangles
        assert!(pd.lines.num_cells() >= 3); // axis lines
    }
}
