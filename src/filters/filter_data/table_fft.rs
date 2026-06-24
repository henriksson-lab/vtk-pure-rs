//! FFT on Table column data.
//!
//! Computes the Discrete Fourier Transform of a column in a Table,
//! producing magnitude and phase arrays.

use crate::data::{AnyDataArray, DataArray, Table};

/// Compute the FFT of a column in a Table.
///
/// Adds `{column_name}_magnitude` and `{column_name}_phase` columns.
/// Uses the Cooley-Tukey radix-2 FFT (pads to next power of 2).
pub fn table_fft(table: &Table, column_name: &str) -> Table {
    let col = match table.column_by_name(column_name) {
        Some(c) => c,
        None => return table.clone(),
    };

    let n = col.num_tuples();
    if n == 0 {
        return table.clone();
    }

    // Extract real values
    let mut real = Vec::with_capacity(n);
    for i in 0..n {
        let mut v = [0.0f64];
        col.tuple_as_f64(i, &mut v);
        real.push(v[0]);
    }

    // Pad to power of 2
    let fft_n = real.len().next_power_of_two();
    real.resize(fft_n, 0.0);
    let mut imag = vec![0.0f64; fft_n];

    // In-place Cooley-Tukey FFT
    fft_in_place(&mut real, &mut imag);

    // Compute magnitude and phase (only first n points)
    let out_n = n.min(fft_n);
    let mut magnitude = Vec::with_capacity(out_n);
    let mut phase = Vec::with_capacity(out_n);

    for i in 0..out_n {
        let mag = (real[i] * real[i] + imag[i] * imag[i]).sqrt();
        let ph = imag[i].atan2(real[i]);
        magnitude.push(mag);
        phase.push(ph);
    }

    let mut result = table.clone();
    result.add_column(AnyDataArray::F64(DataArray::from_vec(
        &format!("{column_name}_magnitude"),
        magnitude,
        1,
    )));
    result.add_column(AnyDataArray::F64(DataArray::from_vec(
        &format!("{column_name}_phase"),
        phase,
        1,
    )));
    result
}

/// In-place Cooley-Tukey radix-2 FFT.
fn fft_in_place(real: &mut [f64], imag: &mut [f64]) {
    let n = real.len();
    assert!(n.is_power_of_two());

    // Bit-reversal permutation
    let mut j = 0;
    for i in 0..n {
        if i < j {
            real.swap(i, j);
            imag.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    // Butterfly operations
    let mut size = 2;
    while size <= n {
        let half = size / 2;
        let angle_step: f64 = -2.0 * std::f64::consts::PI / size as f64;

        for start in (0..n).step_by(size) {
            let mut angle: f64 = 0.0;
            for k in 0..half {
                let cos = angle.cos();
                let sin = angle.sin();
                let i1 = start + k;
                let i2 = start + k + half;

                let tr = real[i2] * cos - imag[i2] * sin;
                let ti = real[i2] * sin + imag[i2] * cos;

                real[i2] = real[i1] - tr;
                imag[i2] = imag[i1] - ti;
                real[i1] += tr;
                imag[i1] += ti;

                angle += angle_step;
            }
        }
        size <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_dc_signal() {
        let mut table = Table::new();
        // Constant signal: FFT should have all energy at DC (index 0)
        let data = vec![1.0; 8];
        table.add_column(AnyDataArray::F64(DataArray::from_vec("signal", data, 1)));

        let result = table_fft(&table, "signal");
        let mag = result.column_by_name("signal_magnitude").unwrap();
        let mut v = [0.0f64];
        mag.tuple_as_f64(0, &mut v);
        assert!(
            (v[0] - 8.0).abs() < 1e-10,
            "DC magnitude should be 8 for 8 ones"
        );
        // Non-DC should be ~0
        mag.tuple_as_f64(1, &mut v);
        assert!(v[0].abs() < 1e-10);
    }

    #[test]
    fn fft_single_frequency() {
        let mut table = Table::new();
        let n = 16;
        let data: Vec<f64> = (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos())
            .collect();
        table.add_column(AnyDataArray::F64(DataArray::from_vec("signal", data, 1)));

        let result = table_fft(&table, "signal");
        let mag = result.column_by_name("signal_magnitude").unwrap();
        let mut v = [0.0f64];
        // For a single cosine at frequency 1, peaks at index 1 and n-1
        mag.tuple_as_f64(1, &mut v);
        assert!(
            v[0] > 7.0,
            "peak at frequency 1 should be large, got {}",
            v[0]
        );
    }
}
