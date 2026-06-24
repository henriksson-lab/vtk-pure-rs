/// Storage for cell topology using offsets + connectivity arrays.
///
/// Mirrors VTK's `vtkCellArray` design: an offsets array of length `num_cells + 1`
/// and a connectivity array containing the point indices for all cells concatenated.
///
/// For cell `i`, the point indices are `connectivity[offsets[i]..offsets[i+1]]`.
///
/// # Examples
///
/// ```
/// use vtk_pure_rs::data::CellArray;
///
/// let mut cells = CellArray::new();
/// cells.push_cell(&[0, 1, 2]);    // triangle
/// cells.push_cell(&[3, 4, 5, 6]); // quad
/// assert_eq!(cells.num_cells(), 2);
/// assert_eq!(cells.cell(0), &[0, 1, 2]);
/// ```
use std::sync::Arc;

/// Storage for cell topology using offsets + connectivity arrays.
/// Uses Arc<Vec<i64>> for zero-copy clone with copy-on-write semantics.
#[derive(Debug)]
pub struct CellArray {
    offsets: Arc<Vec<i64>>,
    connectivity: Arc<Vec<i64>>,
}

impl Clone for CellArray {
    fn clone(&self) -> Self {
        Self {
            offsets: Arc::clone(&self.offsets),
            connectivity: Arc::clone(&self.connectivity),
        }
    }
}

impl PartialEq for CellArray {
    fn eq(&self, other: &Self) -> bool {
        (Arc::ptr_eq(&self.offsets, &other.offsets) || *self.offsets == *other.offsets)
            && (Arc::ptr_eq(&self.connectivity, &other.connectivity)
                || *self.connectivity == *other.connectivity)
    }
}

impl Default for CellArray {
    fn default() -> Self {
        Self::new()
    }
}

impl CellArray {
    pub fn new() -> Self {
        Self {
            offsets: Arc::new(vec![0]),
            connectivity: Arc::new(Vec::new()),
        }
    }

    /// Create a CellArray from raw offsets and connectivity.
    pub fn from_raw(offsets: Vec<i64>, connectivity: Vec<i64>) -> Self {
        assert!(
            !offsets.is_empty(),
            "offsets must have at least one element"
        );
        assert_eq!(offsets[0], 0, "first offset must be 0");
        let conn_len = connectivity.len() as i64;
        for pair in offsets.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "offsets must be monotonically non-decreasing"
            );
        }
        assert!(
            offsets.iter().all(|&o| o >= 0 && o <= conn_len),
            "offsets must be in range 0..=connectivity.len()"
        );
        assert_eq!(
            *offsets.last().unwrap(),
            conn_len,
            "last offset must equal connectivity length"
        );
        Self {
            offsets: Arc::new(offsets),
            connectivity: Arc::new(connectivity),
        }
    }

    /// Create a CellArray from triangle index arrays.
    pub fn from_triangles(tris: &[[i64; 3]]) -> Self {
        let mut ca = Self::new();
        for tri in tris {
            ca.push_cell(tri);
        }
        ca
    }

    /// Create a CellArray from quad index arrays.
    pub fn from_quads(quads: &[[i64; 4]]) -> Self {
        let mut ca = Self::new();
        for quad in quads {
            ca.push_cell(quad);
        }
        ca
    }

    /// Append a cell with the given point indices.
    pub fn push_cell(&mut self, point_ids: &[i64]) {
        // Fast path: if sole owner, avoid Arc::make_mut's atomic CAS
        if let Some(c) = Arc::get_mut(&mut self.connectivity) {
            c.extend_from_slice(point_ids);
        } else {
            Arc::make_mut(&mut self.connectivity).extend_from_slice(point_ids);
        }
        let conn_len = self.connectivity.len() as i64;
        if let Some(o) = Arc::get_mut(&mut self.offsets) {
            o.push(conn_len);
        } else {
            Arc::make_mut(&mut self.offsets).push(conn_len);
        }
    }

    pub fn num_cells(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    /// Returns the point indices for cell `idx`.
    pub fn cell(&self, idx: usize) -> &[i64] {
        let start = self.offsets[idx] as usize;
        let end = self.offsets[idx + 1] as usize;
        &self.connectivity[start..end]
    }

    pub fn is_empty(&self) -> bool {
        self.num_cells() == 0
    }

    pub fn offsets(&self) -> &[i64] {
        &self.offsets
    }

    pub fn connectivity(&self) -> &[i64] {
        &self.connectivity
    }

    /// Total number of point index entries across all cells.
    pub fn connectivity_len(&self) -> usize {
        self.connectivity.len()
    }

    pub fn clear(&mut self) {
        let off = Arc::make_mut(&mut self.offsets);
        off.clear();
        off.push(0);
        Arc::make_mut(&mut self.connectivity).clear();
    }

    /// Get the number of points in cell at `idx`.
    pub fn cell_size(&self, idx: usize) -> usize {
        let start = self.offsets[idx] as usize;
        let end = self.offsets[idx + 1] as usize;
        end - start
    }

    /// Iterator over cell sizes (number of points per cell).
    pub fn cell_sizes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.num_cells()).map(move |i| self.cell_size(i))
    }

    /// Maximum cell size (max number of points in any cell).
    pub fn max_cell_size(&self) -> usize {
        self.cell_sizes().max().unwrap_or(0)
    }

    /// Check if all cells have the same size.
    pub fn is_homogeneous(&self) -> Option<usize> {
        let mut sizes = self.cell_sizes();
        let first = sizes.next()?;
        if sizes.all(|s| s == first) {
            Some(first)
        } else {
            None
        }
    }

    /// Iterate over cells, yielding a slice of point indices for each.
    pub fn iter(&self) -> CellIter<'_> {
        CellIter {
            cells: self,
            idx: 0,
        }
    }
}

pub struct CellIter<'a> {
    cells: &'a CellArray,
    idx: usize,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = &'a [i64];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.cells.num_cells() {
            return None;
        }
        let cell = self.cells.cell(self.idx);
        self.idx += 1;
        Some(cell)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.cells.num_cells() - self.idx;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CellIter<'_> {}

impl<'a> IntoIterator for &'a CellArray {
    type Item = &'a [i64];
    type IntoIter = CellIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_iterate() {
        let mut cells = CellArray::new();
        cells.push_cell(&[0, 1, 2]);
        cells.push_cell(&[2, 3, 4, 5]);
        assert_eq!(cells.num_cells(), 2);
        assert_eq!(cells.cell(0), &[0, 1, 2]);
        assert_eq!(cells.cell(1), &[2, 3, 4, 5]);

        let collected: Vec<&[i64]> = cells.iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn empty_cell_array() {
        let cells = CellArray::new();
        assert!(cells.is_empty());
        assert_eq!(cells.num_cells(), 0);
        assert_eq!(cells.iter().count(), 0);
    }

    #[test]
    fn from_raw() {
        let cells = CellArray::from_raw(vec![0, 3, 7], vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(cells.num_cells(), 2);
        assert_eq!(cells.cell(0), &[0, 1, 2]);
        assert_eq!(cells.cell(1), &[3, 4, 5, 6]);
    }

    #[test]
    #[should_panic(expected = "last offset must equal connectivity length")]
    fn from_raw_rejects_truncated_offsets() {
        CellArray::from_raw(vec![0, 3], vec![0, 1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "offsets must be monotonically non-decreasing")]
    fn from_raw_rejects_decreasing_offsets() {
        CellArray::from_raw(vec![0, 3, 2], vec![0, 1, 2]);
    }

    #[test]
    fn cell_sizes() {
        let mut cells = CellArray::new();
        cells.push_cell(&[0, 1, 2]);
        cells.push_cell(&[2, 3, 4, 5]);
        assert_eq!(cells.cell_size(0), 3);
        assert_eq!(cells.cell_size(1), 4);
        assert_eq!(cells.max_cell_size(), 4);
        let sizes: Vec<usize> = cells.cell_sizes().collect();
        assert_eq!(sizes, vec![3, 4]);
    }

    #[test]
    fn homogeneous() {
        let mut tris = CellArray::new();
        tris.push_cell(&[0, 1, 2]);
        tris.push_cell(&[3, 4, 5]);
        assert_eq!(tris.is_homogeneous(), Some(3));

        let mut mixed = CellArray::new();
        mixed.push_cell(&[0, 1, 2]);
        mixed.push_cell(&[0, 1, 2, 3]);
        assert_eq!(mixed.is_homogeneous(), None);
    }

    #[test]
    fn into_iterator() {
        let mut cells = CellArray::new();
        cells.push_cell(&[0, 1]);
        cells.push_cell(&[2, 3]);
        let mut count = 0;
        for _cell in &cells {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn from_triangles() {
        let cells = CellArray::from_triangles(&[[0, 1, 2], [3, 4, 5]]);
        assert_eq!(cells.num_cells(), 2);
        assert_eq!(cells.cell(0), &[0, 1, 2]);
        assert_eq!(cells.is_homogeneous(), Some(3));
    }

    #[test]
    fn from_quads() {
        let cells = CellArray::from_quads(&[[0, 1, 2, 3]]);
        assert_eq!(cells.num_cells(), 1);
        assert_eq!(cells.cell(0).len(), 4);
    }
}
