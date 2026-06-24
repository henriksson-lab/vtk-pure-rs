//! Shape complexity metrics (convexity defect, roughness, fractal dimension estimate).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub struct ShapeComplexity {
    pub convexity_ratio: f64,
    pub surface_roughness: f64,
    pub compactness: f64,
    pub fractal_dim_estimate: f64,
}
pub fn shape_complexity(mesh: &PolyData) -> ShapeComplexity {
    let n = mesh.points.len();
    if n < 4 {
        return ShapeComplexity {
            convexity_ratio: 1.0,
            surface_roughness: 0.0,
            compactness: 1.0,
            fractal_dim_estimate: 2.0,
        };
    }
    // Surface area
    let mut area = 0.0;
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx2 = e1[1] * e2[2] - e1[2] * e2[1];
            let cy2 = e1[2] * e2[0] - e1[0] * e2[2];
            let cz2 = e1[0] * e2[1] - e1[1] * e2[0];
            area += 0.5 * (cx2 * cx2 + cy2 * cy2 + cz2 * cz2).sqrt();
            vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
    }
    vol = vol.abs() / 6.0;
    // Compactness (isoperimetric ratio)
    let compact = if area > 1e-15 {
        36.0 * std::f64::consts::PI * vol * vol / (area * area * area)
    } else {
        0.0
    };
    // Bounding box volume
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    let bb_vol = (mx[0] - mn[0]) * (mx[1] - mn[1]) * (mx[2] - mn[2]);
    let convexity = if bb_vol > 1e-15 { vol / bb_vol } else { 0.0 };
    // Roughness (average Laplacian magnitude)
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut roughness = 0.0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let p = mesh.points.get(i);
        let k = nb[i].len() as f64;
        let mut lap = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        roughness += (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k;
    }
    roughness /= n as f64;
    // Fractal dimension estimate via box-counting approximation
    let diag = ((mx[0] - mn[0]).powi(2) + (mx[1] - mn[1]).powi(2) + (mx[2] - mn[2]).powi(2)).sqrt();
    let fractal = if area > 1e-15 && diag > 1e-15 {
        2.0 + (area / (diag * diag) - 1.0).max(0.0).min(1.0) * 0.5
    } else {
        2.0
    };
    ShapeComplexity {
        convexity_ratio: convexity,
        surface_roughness: roughness,
        compactness: compact,
        fractal_dim_estimate: fractal,
    }
}
pub fn attach_complexity_scalars(mesh: &PolyData) -> PolyData {
    let sc = shape_complexity(mesh);
    let n = mesh.points.len();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Convexity",
            vec![sc.convexity_ratio; n],
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Roughness",
            vec![sc.surface_roughness; n],
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Compactness",
            vec![sc.compactness; n],
            1,
        )));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let sc = shape_complexity(&m);
        assert!(sc.compactness > 0.0);
        assert!(sc.fractal_dim_estimate >= 2.0);
    }
    #[test]
    fn test_attach() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_complexity_scalars(&m);
        assert!(r.point_data().get_array("Compactness").is_some());
    }
}
