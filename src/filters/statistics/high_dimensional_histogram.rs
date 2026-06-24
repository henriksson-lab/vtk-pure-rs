//! N-dimensional histogram computation.

use crate::data::{AnyDataArray, DataArray, Table};

/// An N-dimensional histogram.
#[derive(Debug, Clone)]
pub struct NdHistogram {
    /// Number of bins per dimension.
    pub bins_per_dim: Vec<usize>,
    /// Min value per dimension.
    pub mins: Vec<f64>,
    /// Max value per dimension.
    pub maxs: Vec<f64>,
    /// Bin width per dimension.
    pub bin_widths: Vec<f64>,
    /// Flat array of bin counts (row-major).
    pub counts: Vec<usize>,
    /// Total number of samples.
    pub total: usize,
    /// Column names.
    pub column_names: Vec<String>,
}

impl NdHistogram {
    /// Get the count for a specific bin index tuple.
    pub fn get(&self, indices: &[usize]) -> usize {
        let flat = self.flat_index(indices);
        if flat < self.counts.len() {
            self.counts[flat]
        } else {
            0
        }
    }

    /// Convert bin indices to the center coordinate of the bin.
    pub fn bin_center(&self, indices: &[usize]) -> Vec<f64> {
        indices
            .iter()
            .enumerate()
            .map(|(d, &i)| self.mins[d] + (i as f64 + 0.5) * self.bin_widths[d])
            .collect()
    }

    fn flat_index(&self, indices: &[usize]) -> usize {
        let mut flat = 0;
        let mut stride = 1;
        for d in (0..indices.len()).rev() {
            flat += indices[d] * stride;
            stride *= self.bins_per_dim[d];
        }
        flat
    }
}

/// Compute an N-dimensional histogram from scalar columns of a Table.
///
/// Each column becomes one dimension. `bins` specifies the number of bins
/// per dimension (same for all if single value).
pub fn nd_histogram(table: &Table, bins: usize) -> Option<NdHistogram> {
    let mut cols: Vec<(String, Vec<f64>)> = Vec::new();
    for col in table.columns() {
        if col.num_components() != 1 {
            continue;
        }
        let n = col.num_tuples();
        let mut values = Vec::with_capacity(n);
        let mut buf = [0.0f64];
        for i in 0..n {
            col.tuple_as_f64(i, &mut buf);
            values.push(buf[0]);
        }
        cols.push((col.name().to_string(), values));
    }

    if cols.is_empty() {
        return None;
    }
    let ndims = cols.len();
    let n = cols[0].1.len();
    if n == 0 {
        return None;
    }

    let mut mins = Vec::with_capacity(ndims);
    let mut maxs = Vec::with_capacity(ndims);
    for (_, vals) in &cols {
        let mn = vals.iter().cloned().fold(f64::MAX, f64::min);
        let mx = vals.iter().cloned().fold(f64::MIN, f64::max);
        mins.push(mn);
        maxs.push(if (mx - mn).abs() < 1e-15 {
            mn + 1.0
        } else {
            mx
        });
    }

    let bin_widths: Vec<f64> = (0..ndims)
        .map(|d| (maxs[d] - mins[d]) / bins as f64)
        .collect();
    let bins_per_dim = vec![bins; ndims];
    let total_bins: usize = bins_per_dim.iter().product();
    let mut counts = vec![0usize; total_bins];

    for i in 0..n {
        let mut indices = Vec::with_capacity(ndims);
        for d in 0..ndims {
            let v = cols[d].1[i];
            let idx = ((v - mins[d]) / bin_widths[d]) as usize;
            indices.push(idx.min(bins - 1));
        }

        let mut flat = 0;
        let mut stride = 1;
        for d in (0..ndims).rev() {
            flat += indices[d] * stride;
            stride *= bins;
        }
        if flat < total_bins {
            counts[flat] += 1;
        }
    }

    Some(NdHistogram {
        bins_per_dim,
        mins,
        maxs,
        bin_widths,
        counts,
        total: n,
        column_names: cols.iter().map(|(name, _)| name.clone()).collect(),
    })
}

/// Convert a 2D histogram to a Table with columns (x_center, y_center, count).
pub fn histogram_2d_to_table(hist: &NdHistogram) -> Table {
    if hist.bins_per_dim.len() != 2 {
        return Table::new();
    }

    let nx = hist.bins_per_dim[0];
    let ny = hist.bins_per_dim[1];
    let mut x_data = Vec::new();
    let mut y_data = Vec::new();
    let mut count_data = Vec::new();

    for ix in 0..nx {
        for iy in 0..ny {
            let center = hist.bin_center(&[ix, iy]);
            let count = hist.get(&[ix, iy]);
            x_data.push(center[0]);
            y_data.push(center[1]);
            count_data.push(count as f64);
        }
    }

    let mut result = Table::new();
    result.add_column(AnyDataArray::F64(DataArray::from_vec("x", x_data, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("y", y_data, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec(
        "count", count_data, 1,
    )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_2d_histogram() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![0.1, 0.2, 0.8, 0.9, 0.5],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![0.1, 0.2, 0.8, 0.9, 0.5],
                1,
            )));

        let hist = nd_histogram(&table, 2).unwrap();
        assert_eq!(hist.bins_per_dim.len(), 2);
        assert_eq!(hist.total, 5);
        let total_counts: usize = hist.counts.iter().sum();
        assert_eq!(total_counts, 5);
    }

    #[test]
    fn to_table() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![0.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![0.0, 1.0],
                1,
            )));
        let hist = nd_histogram(&table, 4).unwrap();
        let result = histogram_2d_to_table(&hist);
        assert_eq!(result.num_rows(), 16); // 4x4
    }

    #[test]
    fn single_dimension() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            1,
        )));
        let hist = nd_histogram(&table, 5).unwrap();
        assert_eq!(hist.bins_per_dim.len(), 1);
    }

    #[test]
    fn three_dimensions() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![0.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![0.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "z",
                vec![0.0, 1.0],
                1,
            )));
        let hist = nd_histogram(&table, 3).unwrap();
        assert_eq!(hist.counts.len(), 27); // 3^3
    }
}
