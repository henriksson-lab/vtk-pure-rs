use crate::data::{Points, PolyData};

/// Simple cage deformation: move control points and propagate.
///
/// Given original cage points and deformed cage points, interpolates
/// the deformation onto the mesh using inverse-distance weighting.
pub fn cage_deform(
    input: &PolyData,
    cage_original: &[[f64; 3]],
    cage_deformed: &[[f64; 3]],
    power: f64,
) -> PolyData {
    let n = input.points.len();
    let nc = cage_original.len().min(cage_deformed.len());
    if n == 0 || nc == 0 {
        return input.clone();
    }

    let mut points = Points::<f64>::new();
    let power = if power.is_finite() { power } else { 2.0 };

    for i in 0..n {
        let p = input.points.get(i);
        if let Some(j) = (0..nc).find(|&j| squared_distance(p, cage_original[j]) <= 1e-24) {
            points.push(cage_deformed[j]);
            continue;
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;
        let mut total_w = 0.0;

        for j in 0..nc {
            let d2 = squared_distance(p, cage_original[j]);
            let w = if power == 0.0 {
                1.0
            } else {
                1.0 / d2.powf(power * 0.5)
            };
            let disp = [
                cage_deformed[j][0] - cage_original[j][0],
                cage_deformed[j][1] - cage_original[j][1],
                cage_deformed[j][2] - cage_original[j][2],
            ];
            dx += w * disp[0];
            dy += w * disp[1];
            dz += w * disp[2];
            total_w += w;
        }

        if total_w > 1e-15 {
            points.push([
                p[0] + dx / total_w,
                p[1] + dy / total_w,
                p[2] + dz / total_w,
            ]);
        } else {
            points.push(p);
        }
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

fn squared_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_via_cage() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let orig = [[0.0, 0.0, 0.0]];
        let deformed = [[10.0, 0.0, 0.0]];

        let result = cage_deform(&pd, &orig, &deformed, 2.0);
        let p = result.points.get(0);
        assert!(p[0] > 5.0); // moved toward deformed cage point
    }

    #[test]
    fn exact_control_point_maps_to_deformed_control_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);

        let orig = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let deformed = [[10.0, 0.0, 0.0], [1.0, 10.0, 0.0]];

        let result = cage_deform(&pd, &orig, &deformed, 2.0);
        assert_eq!(result.points.get(0), [10.0, 0.0, 0.0]);
    }

    #[test]
    fn no_deformation() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);

        let orig = [[0.0, 0.0, 0.0]];
        let result = cage_deform(&pd, &orig, &orig, 2.0);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn empty_cage() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = cage_deform(&pd, &[], &[], 2.0);
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = cage_deform(&pd, &[[0.0; 3]], &[[1.0, 0.0, 0.0]], 2.0);
        assert_eq!(result.points.len(), 0);
    }
}
