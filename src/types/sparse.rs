//! Sparse matrix in CSR (Compressed Sparse Row) format.

/// A sparse matrix stored in Compressed Sparse Row format.
///
/// Efficient for matrix-vector multiplication and row access.
#[derive(Debug, Clone)]
pub struct SparseMatrix {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Row pointers: row_ptr[i]..row_ptr[i+1] gives the range of entries in row i.
    row_ptr: Vec<usize>,
    /// Column indices for each non-zero entry.
    col_idx: Vec<usize>,
    /// Values for each non-zero entry.
    values: Vec<f64>,
}

impl SparseMatrix {
    /// Create an empty sparse matrix.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            row_ptr: vec![0; rows + 1],
            col_idx: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Build from COO (coordinate) format triplets.
    pub fn from_triplets(rows: usize, cols: usize, triplets: &[(usize, usize, f64)]) -> Self {
        // Sort by row then column
        let mut sorted = triplets.to_vec();
        sorted.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut row_ptr = vec![0usize; rows + 1];
        let mut col_idx = Vec::with_capacity(sorted.len());
        let mut values = Vec::with_capacity(sorted.len());

        for &(r, c, v) in &sorted {
            row_ptr[r + 1] += 1;
            col_idx.push(c);
            values.push(v);
        }

        // Cumulative sum
        for i in 0..rows {
            row_ptr[i + 1] += row_ptr[i];
        }

        Self {
            rows,
            cols,
            row_ptr,
            col_idx,
            values,
        }
    }

    /// Number of non-zero entries.
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// Get value at (row, col). Returns 0 if not stored.
    pub fn get(&self, row: usize, col: usize) -> f64 {
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];
        for i in start..end {
            if self.col_idx[i] == col {
                return self.values[i];
            }
        }
        0.0
    }

    /// Multiply matrix by a dense vector: y = A * x.
    pub fn mul_vec(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.cols);
        let mut y = vec![0.0; self.rows];
        for row in 0..self.rows {
            let start = self.row_ptr[row];
            let end = self.row_ptr[row + 1];
            for i in start..end {
                y[row] += self.values[i] * x[self.col_idx[i]];
            }
        }
        y
    }

    /// Create an identity matrix.
    pub fn identity(n: usize) -> Self {
        let triplets: Vec<(usize, usize, f64)> = (0..n).map(|i| (i, i, 1.0)).collect();
        Self::from_triplets(n, n, &triplets)
    }

    /// Sparsity (fraction of non-zero entries).
    pub fn sparsity(&self) -> f64 {
        if self.rows == 0 || self.cols == 0 {
            return 0.0;
        }
        1.0 - self.nnz() as f64 / (self.rows * self.cols) as f64
    }
}

impl std::fmt::Display for SparseMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SparseMatrix({}x{}, {} nnz, {:.1}% sparse)",
            self.rows,
            self.cols,
            self.nnz(),
            self.sparsity() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let m = SparseMatrix::identity(3);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(0, 1), 0.0);
        assert_eq!(m.get(1, 1), 1.0);
        assert_eq!(m.nnz(), 3);
    }

    #[test]
    fn from_triplets() {
        let m = SparseMatrix::from_triplets(
            3,
            3,
            &[(0, 0, 2.0), (0, 1, 1.0), (1, 1, 3.0), (2, 2, 4.0)],
        );
        assert_eq!(m.get(0, 0), 2.0);
        assert_eq!(m.get(1, 1), 3.0);
        assert_eq!(m.get(2, 0), 0.0);
    }

    #[test]
    fn mul_vec() {
        let m = SparseMatrix::identity(3);
        let x = vec![1.0, 2.0, 3.0];
        let y = m.mul_vec(&x);
        assert_eq!(y, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn mul_vec_general() {
        let m = SparseMatrix::from_triplets(
            2,
            2,
            &[(0, 0, 2.0), (0, 1, 1.0), (1, 0, 0.0), (1, 1, 3.0)],
        );
        let y = m.mul_vec(&[1.0, 2.0]);
        assert!((y[0] - 4.0).abs() < 1e-10); // 2*1 + 1*2
        assert!((y[1] - 6.0).abs() < 1e-10); // 0*1 + 3*2
    }

    #[test]
    fn display() {
        let m = SparseMatrix::identity(100);
        let s = format!("{m}");
        assert!(s.contains("100x100"));
        assert!(s.contains("100 nnz"));
    }
}
