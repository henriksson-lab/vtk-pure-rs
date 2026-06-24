//! Implicit array compression filters.
//!
//! Compress explicit data arrays into compact implicit representations:
//! - Constant arrays (all values identical)
//! - Affine arrays (start + step * index)
//! - Ramer-Douglas-Peucker simplified polylines

use crate::data::DataArray;

/// Result of attempting to compress a data array.
#[derive(Debug, Clone)]
pub enum ImplicitArray {
    /// All values are the same constant.
    Constant { value: f64, count: usize },
    /// Values follow an affine pattern: value[i] = start + step * i.
    Affine { start: f64, step: f64, count: usize },
    /// Could not be represented implicitly — keep explicit.
    Explicit(DataArray<f64>),
}

impl ImplicitArray {
    /// Reconstruct explicit values.
    pub fn to_explicit(&self, name: &str) -> DataArray<f64> {
        match self {
            ImplicitArray::Constant { value, count } => {
                DataArray::from_vec(name, vec![*value; *count], 1)
            }
            ImplicitArray::Affine { start, step, count } => {
                let data: Vec<f64> = (0..*count).map(|i| start + step * i as f64).collect();
                DataArray::from_vec(name, data, 1)
            }
            ImplicitArray::Explicit(arr) => arr.clone(),
        }
    }

    /// Memory ratio: implicit size / explicit size.
    pub fn compression_ratio(&self) -> f64 {
        match self {
            ImplicitArray::Constant { count, .. } => 1.0 / *count as f64,
            ImplicitArray::Affine { count, .. } => 2.0 / *count as f64,
            ImplicitArray::Explicit(_) => 1.0,
        }
    }
}

/// Try to compress a data array to an implicit representation.
///
/// Checks for constant and affine patterns within the given tolerance.
pub fn to_implicit_array(input: &DataArray<f64>, tolerance: f64) -> ImplicitArray {
    let n = input.num_tuples();
    if n == 0 {
        return ImplicitArray::Constant {
            value: 0.0,
            count: 0,
        };
    }
    if input.num_components() != 1 {
        return ImplicitArray::Explicit(input.clone());
    }

    let first = input.tuple(0)[0];

    // Check constant
    let is_constant = (1..n).all(|i| (input.tuple(i)[0] - first).abs() <= tolerance);
    if is_constant {
        return ImplicitArray::Constant {
            value: first,
            count: n,
        };
    }

    // Check affine (linear)
    if n >= 2 {
        let step = input.tuple(1)[0] - first;
        let is_affine = (2..n).all(|i| {
            let expected = first + step * i as f64;
            (input.tuple(i)[0] - expected).abs() <= tolerance
        });
        if is_affine {
            return ImplicitArray::Affine {
                start: first,
                step,
                count: n,
            };
        }
    }

    ImplicitArray::Explicit(input.clone())
}

/// Ramer-Douglas-Peucker polyline simplification.
///
/// Reduces the number of points in a polyline while preserving shape within `epsilon`.
/// Returns indices of the kept points.
pub fn ramer_douglas_peucker(points: &[[f64; 3]], epsilon: f64) -> Vec<usize> {
    if points.len() <= 2 {
        return (0..points.len()).collect();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    rdp_recursive(points, 0, points.len() - 1, epsilon, &mut keep);
    keep.iter()
        .enumerate()
        .filter(|(_, &k)| k)
        .map(|(i, _)| i)
        .collect()
}

fn rdp_recursive(points: &[[f64; 3]], start: usize, end: usize, epsilon: f64, keep: &mut [bool]) {
    if end <= start + 1 {
        return;
    }

    let mut max_dist = 0.0;
    let mut max_idx = start;

    let a = points[start];
    let b = points[end];

    for i in (start + 1)..end {
        let d = point_to_line_distance(points[i], a, b);
        if d > max_dist {
            max_dist = d;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        keep[max_idx] = true;
        rdp_recursive(points, start, max_idx, epsilon, keep);
        rdp_recursive(points, max_idx, end, epsilon, keep);
    }
}

fn point_to_line_distance(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let ab_len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];

    if ab_len2 < 1e-30 {
        return (ap[0] * ap[0] + ap[1] * ap[1] + ap[2] * ap[2]).sqrt();
    }

    let t = ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab_len2).clamp(0.0, 1.0);
    let closest = [a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]];
    let d = [p[0] - closest[0], p[1] - closest[1], p[2] - closest[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_constant() {
        let arr = DataArray::from_vec("c", vec![5.0; 100], 1);
        let result = to_implicit_array(&arr, 1e-10);
        assert!(matches!(result, ImplicitArray::Constant { .. }));
        assert!(result.compression_ratio() < 0.1);
    }

    #[test]
    fn detect_affine() {
        let data: Vec<f64> = (0..100).map(|i| 2.0 + 0.5 * i as f64).collect();
        let arr = DataArray::from_vec("a", data, 1);
        let result = to_implicit_array(&arr, 1e-10);
        match result {
            ImplicitArray::Affine { start, step, count } => {
                assert!((start - 2.0).abs() < 1e-10);
                assert!((step - 0.5).abs() < 1e-10);
                assert_eq!(count, 100);
            }
            _ => panic!("expected affine"),
        }
    }

    #[test]
    fn explicit_fallback() {
        let arr = DataArray::from_vec("e", vec![1.0, 3.0, 2.0, 7.0], 1);
        let result = to_implicit_array(&arr, 1e-10);
        assert!(matches!(result, ImplicitArray::Explicit(_)));
    }

    #[test]
    fn rdp_simplification() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [0.5, 0.01, 0.0], // nearly on line
            [1.0, 0.0, 0.0],
            [1.5, 0.01, 0.0], // nearly on line
            [2.0, 0.0, 0.0],
        ];
        let kept = ramer_douglas_peucker(&points, 0.1);
        assert_eq!(kept, vec![0, 4]); // only endpoints kept
    }

    #[test]
    fn rdp_preserves_corners() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0], // sharp corner
            [2.0, 1.0, 0.0],
        ];
        let kept = ramer_douglas_peucker(&points, 0.1);
        assert!(kept.contains(&2), "corner should be preserved");
    }

    #[test]
    fn roundtrip_affine() {
        let data: Vec<f64> = (0..50).map(|i| 10.0 + 3.0 * i as f64).collect();
        let arr = DataArray::from_vec("a", data.clone(), 1);
        let implicit = to_implicit_array(&arr, 1e-10);
        let explicit = implicit.to_explicit("a");
        for i in 0..50 {
            assert!((explicit.tuple(i)[0] - data[i]).abs() < 1e-10);
        }
    }
}
