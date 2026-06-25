//! Comprehensive image statistics summary.

use crate::data::ImageData;

/// Image statistics summary.
#[derive(Debug, Clone)]
pub struct ImageStats {
    pub dimensions: [usize; 3],
    pub spacing: [f64; 3],
    pub origin: [f64; 3],
    pub num_points: usize,
    pub num_arrays: usize,
    pub array_stats: Vec<ArrayStats>,
}

/// Per-array statistics.
#[derive(Debug, Clone)]
pub struct ArrayStats {
    pub name: String,
    pub components: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std: f64,
    pub non_zero_count: usize,
}

impl std::fmt::Display for ImageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Image: {}×{}×{}, spacing=[{:.3},{:.3},{:.3}], origin=[{:.3},{:.3},{:.3}]",
            self.dimensions[0],
            self.dimensions[1],
            self.dimensions[2],
            self.spacing[0],
            self.spacing[1],
            self.spacing[2],
            self.origin[0],
            self.origin[1],
            self.origin[2]
        )?;
        writeln!(
            f,
            "Points: {}, Arrays: {}",
            self.num_points, self.num_arrays
        )?;
        for a in &self.array_stats {
            writeln!(
                f,
                "  {}: {}comp, range=[{:.4},{:.4}], mean={:.4}, std={:.4}, nonzero={}",
                a.name, a.components, a.min, a.max, a.mean, a.std, a.non_zero_count
            )?;
        }
        Ok(())
    }
}

/// Compute comprehensive statistics for an ImageData.
pub fn image_stats(image: &ImageData) -> ImageStats {
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let pd = image.point_data();

    let mut array_stats = Vec::new();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let nt = arr.num_tuples();
            let mut buf = vec![0.0f64; nc];
            let mut min_v = f64::MAX;
            let mut max_v = f64::MIN;
            let mut sum = 0.0;
            let mut sum2 = 0.0;
            let mut nonzero = 0;

            for i in 0..nt {
                arr.tuple_as_f64(i, &mut buf);
                let v = buf[0]; // use first component for scalar stats
                min_v = min_v.min(v);
                max_v = max_v.max(v);
                sum += v;
                sum2 += v * v;
                if v.abs() > 1e-15 {
                    nonzero += 1;
                }
            }
            let mean = if nt > 0 { sum / nt as f64 } else { 0.0 };
            let std = if nt > 0 {
                ((sum2 / nt as f64) - mean * mean).max(0.0).sqrt()
            } else {
                0.0
            };

            array_stats.push(ArrayStats {
                name: arr.name().to_string(),
                components: nc,
                min: min_v,
                max: max_v,
                mean,
                std,
                non_zero_count: nonzero,
            });
        }
    }

    ImageStats {
        dimensions: dims,
        spacing: image.spacing(),
        origin: image.origin(),
        num_points: n,
        num_arrays: pd.num_arrays(),
        array_stats,
    }
}

/// Quick one-line summary.
pub fn image_summary_string(image: &ImageData) -> String {
    let dims = image.dimensions();
    let sp = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    format!(
        "{}×{}×{} ({} pts), spacing=[{:.3},{:.3},{:.3}], {} arrays",
        dims[0],
        dims[1],
        dims[2],
        n,
        sp[0],
        sp[1],
        sp[2],
        image.point_data().num_arrays()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let s = image_stats(&img);
        assert_eq!(s.dimensions, [5, 5, 5]);
        assert_eq!(s.num_arrays, 1);
        assert!(!s.array_stats.is_empty());
    }
    #[test]
    fn display() {
        let img = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let s = format!("{}", image_stats(&img));
        assert!(s.contains("Image:"));
    }
    #[test]
    fn summary() {
        let img = ImageData::with_dimensions(10, 10, 10);
        let s = image_summary_string(&img);
        assert!(s.contains("10×10×10"));
    }
}
