use crate::data::PolyData;
use crate::filters::core::mesh::aspect_ratio::{triangle_aspect_ratio, VERDICT_DBL_MAX};

/// Comprehensive aspect ratio analysis for mesh quality assessment.
#[derive(Debug, Clone)]
pub struct AspectRatioReport {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub pct_good: f64, // fraction with ratio < 2
    pub pct_fair: f64, // fraction with ratio 2-5
    pub pct_poor: f64, // fraction with ratio > 5
    pub num_degenerate: usize,
}

/// Generate aspect ratio quality report.
pub fn aspect_ratio_report(input: &PolyData) -> AspectRatioReport {
    let mut ratios = Vec::new();
    let mut degenerate = 0;

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            degenerate += 1;
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let ratio = triangle_aspect_ratio(v0, v1, v2);
        if ratio >= VERDICT_DBL_MAX {
            degenerate += 1;
            continue;
        }
        ratios.push(ratio);
    }

    if ratios.is_empty() {
        return AspectRatioReport {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            pct_good: 0.0,
            pct_fair: 0.0,
            pct_poor: 0.0,
            num_degenerate: degenerate,
        };
    }

    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = ratios.len();
    let sum: f64 = ratios.iter().sum();
    let mean = sum / n as f64;
    let var: f64 = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n as f64;
    let median = if n % 2 == 1 {
        ratios[n / 2]
    } else {
        (ratios[n / 2 - 1] + ratios[n / 2]) * 0.5
    };
    let good = ratios.iter().filter(|&&r| r < 2.0).count() as f64 / n as f64;
    let fair = ratios.iter().filter(|&&r| r >= 2.0 && r < 5.0).count() as f64 / n as f64;
    let poor = ratios.iter().filter(|&&r| r >= 5.0).count() as f64 / n as f64;

    AspectRatioReport {
        min: ratios[0],
        max: ratios[n - 1],
        mean,
        median,
        std_dev: var.sqrt(),
        pct_good: good,
        pct_fair: fair,
        pct_poor: poor,
        num_degenerate: degenerate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_good() {
        let h = (3.0f64).sqrt() / 2.0;
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let r = aspect_ratio_report(&pd);
        assert!((r.min - 1.0).abs() < 0.01);
        assert_eq!(r.pct_good, 1.0);
    }

    #[test]
    fn mixed_quality() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.01, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let r = aspect_ratio_report(&pd);
        assert!(r.max > r.min);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let r = aspect_ratio_report(&pd);
        assert_eq!(r.mean, 0.0);
    }
}
