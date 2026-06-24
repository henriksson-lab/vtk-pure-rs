use crate::data::PolyData;

/// Validation issues found in a PolyData mesh.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub degenerate_cells: usize,
    pub out_of_range_indices: usize,
    pub duplicate_points: usize,
    pub empty_cells: usize,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.degenerate_cells == 0 && self.out_of_range_indices == 0 && self.empty_cells == 0
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_valid() {
            write!(f, "Valid")
        } else {
            write!(
                f,
                "Invalid: {} degenerate, {} OOB, {} empty",
                self.degenerate_cells, self.out_of_range_indices, self.empty_cells
            )
        }
    }
}

/// Validate a PolyData mesh for common issues.
pub fn validate(pd: &PolyData) -> ValidationReport {
    let mut report = ValidationReport::default();
    let n_pts = pd.points.len();

    for cell in pd.polys.iter() {
        if cell.is_empty() {
            report.empty_cells += 1;
            continue;
        }
        if cell.len() < 3 {
            report.degenerate_cells += 1;
        }
        for &pid in cell {
            if pid < 0 || (pid as usize) >= n_pts {
                report.out_of_range_indices += 1;
            }
        }
    }

    for cell in pd.lines.iter() {
        if cell.is_empty() {
            report.empty_cells += 1;
        }
        if cell.len() < 2 {
            report.degenerate_cells += 1;
        }
        for &pid in cell {
            if pid < 0 || (pid as usize) >= n_pts {
                report.out_of_range_indices += 1;
            }
        }
    }

    // Check for duplicate points using sorting-based approach (O(n log n))
    if n_pts <= 100_000 {
        let pts = pd.points.as_flat_slice();
        // Sort point indices by quantized position for fast duplicate detection
        let mut indices: Vec<u32> = (0..n_pts as u32).collect();
        // Quantize to ~1e-10 resolution and sort by (x, y, z) as integer keys
        let scale = 1e10;
        indices.sort_unstable_by(|&a, &b| {
            let ab = a as usize * 3;
            let bb = b as usize * 3;
            let ax = (pts[ab] * scale) as i64;
            let bx = (pts[bb] * scale) as i64;
            ax.cmp(&bx)
                .then_with(|| ((pts[ab + 1] * scale) as i64).cmp(&((pts[bb + 1] * scale) as i64)))
                .then_with(|| ((pts[ab + 2] * scale) as i64).cmp(&((pts[bb + 2] * scale) as i64)))
        });

        let mut dupes = 0usize;
        for w in indices.windows(2) {
            let ab = w[0] as usize * 3;
            let bb = w[1] as usize * 3;
            let d2 = (pts[ab] - pts[bb]) * (pts[ab] - pts[bb])
                + (pts[ab + 1] - pts[bb + 1]) * (pts[ab + 1] - pts[bb + 1])
                + (pts[ab + 2] - pts[bb + 2]) * (pts[ab + 2] - pts[bb + 2]);
            if d2 < 1e-20 {
                dupes += 1;
            }
        }
        report.duplicate_points = dupes;
    }

    if report.duplicate_points > 0 {
        report.warnings.push(format!(
            "{} duplicate points detected",
            report.duplicate_points
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_mesh() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = validate(&pd);
        assert!(report.is_valid());
    }

    #[test]
    fn duplicate_points() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = validate(&pd);
        assert!(report.duplicate_points > 0);
    }

    #[test]
    fn display() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = validate(&pd);
        assert_eq!(format!("{report}"), "Valid");
    }
}
