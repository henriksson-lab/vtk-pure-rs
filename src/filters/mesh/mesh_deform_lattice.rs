//! Lattice-based deformation with control points.
use crate::data::PolyData;
pub fn deform_1d_lattice(
    mesh: &PolyData,
    axis: usize,
    control_points: &[(f64, [f64; 3])],
) -> PolyData {
    if control_points.is_empty() {
        return mesh.clone();
    }
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let t = p[axis];
        // Find surrounding control points
        let mut below = None::<usize>;
        let mut above = None::<usize>;
        for (ci, &(ct, _)) in control_points.iter().enumerate() {
            if ct <= t {
                match below {
                    None => below = Some(ci),
                    Some(bi) => {
                        if ct > control_points[bi].0 {
                            below = Some(ci);
                        }
                    }
                }
            }
            if ct >= t {
                match above {
                    None => above = Some(ci),
                    Some(ai) => {
                        if ct < control_points[ai].0 {
                            above = Some(ci);
                        }
                    }
                }
            }
        }
        let disp = match (below, above) {
            (Some(bi), Some(ai)) if bi != ai => {
                let (t0, d0) = control_points[bi];
                let (t1, d1) = control_points[ai];
                let frac = (t - t0) / (t1 - t0).max(1e-15);
                [
                    d0[0] * (1.0 - frac) + d1[0] * frac,
                    d0[1] * (1.0 - frac) + d1[1] * frac,
                    d0[2] * (1.0 - frac) + d1[2] * frac,
                ]
            }
            (Some(bi), _) => control_points[bi].1,
            (_, Some(ai)) => control_points[ai].1,
            _ => [0.0, 0.0, 0.0],
        };
        r.points
            .set(i, [p[0] + disp[0], p[1] + disp[1], p[2] + disp[2]]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = deform_1d_lattice(&m, 2, &[(0.0, [0.0, 0.0, 0.0]), (1.0, [1.0, 0.0, 0.0])]);
        let p = r.points.get(2);
        assert!((p[0] - 1.5).abs() < 1e-10);
    }
}
