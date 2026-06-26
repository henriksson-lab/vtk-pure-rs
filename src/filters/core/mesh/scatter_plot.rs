use crate::data::{CellArray, Points, PolyData};

/// Create a 3D scatter plot PolyData from two or three scalar arrays.
///
/// Maps scalar values to X, Y, (optionally Z) coordinates.
/// Useful for visualizing correlations between data arrays.
pub fn scatter_plot_3d(
    input: &PolyData,
    x_array: &str,
    y_array: &str,
    z_array: Option<&str>,
) -> PolyData {
    let xa = match input.point_data().get_array(x_array) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let ya = match input.point_data().get_array(y_array) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let za = z_array.and_then(|name| input.point_data().get_array(name));

    let n = if let Some(a) = &za {
        xa.num_tuples().min(ya.num_tuples()).min(a.num_tuples())
    } else {
        xa.num_tuples().min(ya.num_tuples())
    };
    let mut xb = [0.0f64];
    let mut yb = [0.0f64];
    let mut zb = [0.0f64];

    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for i in 0..n {
        xa.tuple_as_f64(i, &mut xb);
        ya.tuple_as_f64(i, &mut yb);
        let z = if let Some(a) = &za {
            a.tuple_as_f64(i, &mut zb);
            zb[0]
        } else {
            0.0
        };
        let idx = out_pts.len() as i64;
        out_pts.push([xb[0], yb[0], z]);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

/// Create a 2D histogram (heatmap) from two scalar arrays.
///
/// Returns (nx, ny, counts) where counts[j*nx+i] is the bin count.
pub fn histogram_2d(
    input: &PolyData,
    x_array: &str,
    y_array: &str,
    nx_bins: usize,
    ny_bins: usize,
) -> (usize, usize, Vec<usize>) {
    let xa = match input.point_data().get_array(x_array) {
        Some(a) => a,
        None => return (0, 0, vec![]),
    };
    let ya = match input.point_data().get_array(y_array) {
        Some(a) => a,
        None => return (0, 0, vec![]),
    };
    let n = xa.num_tuples().min(ya.num_tuples());
    let nbx = nx_bins.max(1);
    let nby = ny_bins.max(1);

    let mut xb = [0.0f64];
    let mut yb = [0.0f64];
    let mut xmin = f64::MAX;
    let mut xmax = f64::MIN;
    let mut ymin = f64::MAX;
    let mut ymax = f64::MIN;
    for i in 0..n {
        xa.tuple_as_f64(i, &mut xb);
        ya.tuple_as_f64(i, &mut yb);
        xmin = xmin.min(xb[0]);
        xmax = xmax.max(xb[0]);
        ymin = ymin.min(yb[0]);
        ymax = ymax.max(yb[0]);
    }

    let xrange = (xmax - xmin).max(1e-15);
    let yrange = (ymax - ymin).max(1e-15);
    let mut counts = vec![0usize; nbx * nby];

    for i in 0..n {
        xa.tuple_as_f64(i, &mut xb);
        ya.tuple_as_f64(i, &mut yb);
        let bx = ((xb[0] - xmin) / xrange * nbx as f64).floor() as usize;
        let by = ((yb[0] - ymin) / yrange * nby as f64).floor() as usize;
        counts[by.min(nby - 1) * nbx + bx.min(nbx - 1)] += 1;
    }

    (nbx, nby, counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn scatter_basic() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([0.0; 3]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![5.0, 4.0, 3.0, 2.0, 1.0],
                1,
            )));

        let result = scatter_plot_3d(&pd, "x", "y", None);
        assert_eq!(result.points.len(), 5);
        let p = result.points.get(0);
        assert_eq!(p, [1.0, 5.0, 0.0]);
    }

    #[test]
    fn histogram_2d_basic() {
        let mut pd = PolyData::new();
        for i in 0..100 {
            pd.points.push([0.0; 3]);
        }
        let x: Vec<f64> = (0..100).map(|i| (i % 10) as f64).collect();
        let y: Vec<f64> = (0..100).map(|i| (i / 10) as f64).collect();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("x", x, 1)));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("y", y, 1)));

        let (nbx, nby, counts) = histogram_2d(&pd, "x", "y", 5, 5);
        assert_eq!(nbx, 5);
        assert_eq!(nby, 5);
        let total: usize = counts.iter().sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert_eq!(scatter_plot_3d(&pd, "x", "y", None).points.len(), 0);
    }

    #[test]
    fn scatter_caps_to_z_array_length() {
        let mut pd = PolyData::new();
        for _ in 0..3 {
            pd.points.push([0.0; 3]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![4.0, 5.0, 6.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("z", vec![7.0], 1)));

        let result = scatter_plot_3d(&pd, "x", "y", Some("z"));
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.points.get(0), [1.0, 4.0, 7.0]);
    }
}
