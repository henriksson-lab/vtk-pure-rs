//! Compute histogram of a point/cell data array.
use crate::data::PolyData;
pub struct Histogram {
    pub bins: Vec<usize>,
    pub min: f64,
    pub max: f64,
    pub bin_width: f64,
}
pub fn point_data_histogram(
    mesh: &PolyData,
    array_name: &str,
    num_bins: usize,
) -> Option<Histogram> {
    let arr = mesh.point_data().get_array(array_name)?;
    if arr.num_components() != 1 {
        return None;
    }
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    build_histogram(&vals, num_bins)
}
pub fn cell_data_histogram(
    mesh: &PolyData,
    array_name: &str,
    num_bins: usize,
) -> Option<Histogram> {
    let arr = mesh.cell_data().get_array(array_name)?;
    if arr.num_components() != 1 {
        return None;
    }
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    build_histogram(&vals, num_bins)
}
fn build_histogram(vals: &[f64], num_bins: usize) -> Option<Histogram> {
    if vals.is_empty() {
        return None;
    }
    let nb = num_bins.max(1);
    let mut mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if mn == mx {
        mn -= 0.5;
        mx += 0.5;
    }
    let range = mx - mn;
    let bw = range / nb as f64;
    let mut bins = vec![0usize; nb];
    for &v in vals {
        let bi = (((v - mn) / range * nb as f64).floor() as usize).min(nb - 1);
        bins[bi] += 1;
    }
    Some(Histogram {
        bins,
        min: mn,
        max: mx,
        bin_width: bw,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let h = point_data_histogram(&m, "s", 5).unwrap();
        assert_eq!(h.bins.len(), 5);
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
    }

    #[test]
    fn empty_array_returns_none() {
        let mut m = PolyData::from_points(Vec::new());
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", Vec::new(), 1)));
        assert!(point_data_histogram(&m, "s", 5).is_none());
    }

    #[test]
    fn constant_array_has_finite_range() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 3]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![2.0, 2.0, 2.0],
                1,
            )));
        let h = point_data_histogram(&m, "s", 4).unwrap();
        assert!(h.min.is_finite());
        assert!(h.max.is_finite());
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
    }
}
