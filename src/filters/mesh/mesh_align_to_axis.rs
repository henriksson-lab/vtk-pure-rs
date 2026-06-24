//! Align mesh principal axis to coordinate axis.
use crate::data::PolyData;
pub fn align_longest_to_x(mesh: &PolyData) -> PolyData {
    align_to(mesh, 0)
}
pub fn align_longest_to_y(mesh: &PolyData) -> PolyData {
    align_to(mesh, 1)
}
pub fn align_longest_to_z(mesh: &PolyData) -> PolyData {
    align_to(mesh, 2)
}
fn align_to(mesh: &PolyData, target_axis: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 2 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    // Find axis of maximum extent
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for a in 0..3 {
            for b in 0..3 {
                cov[a][b] += d[a] * d[b];
            }
        }
    }
    // Power iteration for principal axis
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..50 {
        let mv = [
            cov[0][0] * v[0] + cov[0][1] * v[1] + cov[0][2] * v[2],
            cov[1][0] * v[0] + cov[1][1] * v[1] + cov[1][2] * v[2],
            cov[2][0] * v[0] + cov[2][1] * v[1] + cov[2][2] * v[2],
        ];
        let l = (mv[0] * mv[0] + mv[1] * mv[1] + mv[2] * mv[2])
            .sqrt()
            .max(1e-15);
        v = [mv[0] / l, mv[1] / l, mv[2] / l];
    }
    // Build rotation that maps v to target axis
    let target = [
        if target_axis == 0 { 1.0 } else { 0.0 },
        if target_axis == 1 { 1.0 } else { 0.0 },
        if target_axis == 2 { 1.0 } else { 0.0 },
    ];
    let dot = v[0] * target[0] + v[1] * target[1] + v[2] * target[2];
    if dot.abs() > 0.999 {
        let mut r = mesh.clone();
        for i in 0..n {
            let p = r.points.get(i);
            r.points.set(i, [p[0] - cx, p[1] - cy, p[2] - cz]);
        }
        return r;
    }
    let cross = [
        v[1] * target[2] - v[2] * target[1],
        v[2] * target[0] - v[0] * target[2],
        v[0] * target[1] - v[1] * target[0],
    ];
    let sl = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let angle = sl.atan2(dot);
    let c = angle.cos();
    let s = angle.sin();
    let t = 1.0 - c;
    let ax = if sl > 1e-15 {
        [cross[0] / sl, cross[1] / sl, cross[2] / sl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let rot = [
        [
            t * ax[0] * ax[0] + c,
            t * ax[0] * ax[1] - s * ax[2],
            t * ax[0] * ax[2] + s * ax[1],
        ],
        [
            t * ax[0] * ax[1] + s * ax[2],
            t * ax[1] * ax[1] + c,
            t * ax[1] * ax[2] - s * ax[0],
        ],
        [
            t * ax[0] * ax[2] - s * ax[1],
            t * ax[1] * ax[2] + s * ax[0],
            t * ax[2] * ax[2] + c,
        ],
    ];
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        r.points.set(
            i,
            [
                rot[0][0] * d[0] + rot[0][1] * d[1] + rot[0][2] * d[2],
                rot[1][0] * d[0] + rot[1][1] * d[1] + rot[1][2] * d[2],
                rot[2][0] * d[0] + rot[2][1] * d[1] + rot[2][2] * d[2],
            ],
        );
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_x() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.0, 0.0, 10.0], [1.0, 0.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = align_longest_to_x(&m);
        // After alignment, extent along X should be largest
        let mut mn = [f64::INFINITY; 3];
        let mut mx = [f64::NEG_INFINITY; 3];
        for i in 0..3 {
            let p = r.points.get(i);
            for j in 0..3 {
                mn[j] = mn[j].min(p[j]);
                mx[j] = mx[j].max(p[j]);
            }
        }
        assert!((mx[0] - mn[0]) >= (mx[1] - mn[1]) * 0.9);
    }
}
