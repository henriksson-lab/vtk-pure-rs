use crate::data::{Points, PolyData};

/// Cotangent-weighted Laplacian smoothing.
///
/// Unlike uniform Laplacian which treats all neighbors equally,
/// cotangent weights account for triangle shape, producing more
/// geometrically accurate smoothing. Better at preserving features.
pub fn cotangent_smooth(input: &PolyData, lambda: f64, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let tris: Vec<[usize; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();

    let pts_init: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut pts = pts_init.clone();

    for _ in 0..iterations {
        // Accumulate cotangent-weighted Laplacian
        let mut lap = vec![[0.0f64; 3]; n];
        let mut weights = vec![0.0f64; n];

        for &[a, b, c] in &tris {
            // For edge a-b, opposite vertex is c
            add_cot_weight(&pts, a, b, c, &mut lap, &mut weights);
            add_cot_weight(&pts, b, c, a, &mut lap, &mut weights);
            add_cot_weight(&pts, c, a, b, &mut lap, &mut weights);
        }

        let mut new_pts = pts.clone();
        for i in 0..n {
            if weights[i] > 1e-15 {
                new_pts[i][0] += lambda * lap[i][0] / weights[i];
                new_pts[i][1] += lambda * lap[i][1] / weights[i];
                new_pts[i][2] += lambda * lap[i][2] / weights[i];
            }
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

fn add_cot_weight(
    pts: &[[f64; 3]],
    i: usize,
    j: usize,
    opp: usize,
    lap: &mut [[f64; 3]],
    w: &mut [f64],
) {
    // Cotangent of angle at opp for edge i-j
    let ei = [
        pts[i][0] - pts[opp][0],
        pts[i][1] - pts[opp][1],
        pts[i][2] - pts[opp][2],
    ];
    let ej = [
        pts[j][0] - pts[opp][0],
        pts[j][1] - pts[opp][1],
        pts[j][2] - pts[opp][2],
    ];
    let dot = ei[0] * ej[0] + ei[1] * ej[1] + ei[2] * ej[2];
    let cross = [
        ei[1] * ej[2] - ei[2] * ej[1],
        ei[2] * ej[0] - ei[0] * ej[2],
        ei[0] * ej[1] - ei[1] * ej[0],
    ];
    let sin_a = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let cot = if sin_a > 1e-15 { dot / sin_a } else { 0.0 };
    let cot = cot.clamp(-10.0, 10.0); // clamp for stability

    let d = [
        pts[j][0] - pts[i][0],
        pts[j][1] - pts[i][1],
        pts[j][2] - pts[i][2],
    ];
    lap[i][0] += cot * d[0];
    lap[i][1] += cot * d[1];
    lap[i][2] += cot * d[2];
    w[i] += cot;
    let d2 = [
        pts[i][0] - pts[j][0],
        pts[i][1] - pts[j][1],
        pts[i][2] - pts[j][2],
    ];
    lap[j][0] += cot * d2[0];
    lap[j][1] += cot * d2[1];
    lap[j][2] += cot * d2[2];
    w[j] += cot;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooths_bump() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 2.0]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 0, 3]);

        let result = cotangent_smooth(&pd, 0.3, 3);
        // Just verify it runs and produces valid output
        assert_eq!(result.points.len(), 4);
        let spike = result.points.get(3);
        assert!(spike[2].is_finite());
    }

    #[test]
    fn zero_lambda_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.points.push([7.0, 8.0, 9.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cotangent_smooth(&pd, 0.0, 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = cotangent_smooth(&pd, 0.5, 10);
        assert_eq!(result.points.len(), 0);
    }
}
