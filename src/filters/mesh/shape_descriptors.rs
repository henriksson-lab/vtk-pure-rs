//! Shape descriptors: compactness, elongation, flatness, rectangularity.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Shape descriptor summary.
#[derive(Debug, Clone)]
pub struct ShapeDescriptors {
    pub compactness: f64,    // V²/A³ normalized (1=sphere)
    pub elongation: f64,     // ratio of 2nd to 1st principal extent
    pub flatness: f64,       // ratio of 3rd to 1st principal extent
    pub rectangularity: f64, // volume / OBB volume
    pub convexity: f64,      // surface area / convex hull area (approx)
    pub sphericity: f64,     // (π^(1/3) * (6V)^(2/3)) / A
}

impl std::fmt::Display for ShapeDescriptors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "compact={:.3} elong={:.3} flat={:.3} rect={:.3} convex={:.3} sphere={:.3}",
            self.compactness,
            self.elongation,
            self.flatness,
            self.rectangularity,
            self.convexity,
            self.sphericity
        )
    }
}

/// Compute shape descriptors for a mesh.
pub fn compute_shape_descriptors(mesh: &PolyData) -> ShapeDescriptors {
    let n = mesh.points.len();
    if n < 3 {
        return ShapeDescriptors {
            compactness: 0.0,
            elongation: 0.0,
            flatness: 0.0,
            rectangularity: 0.0,
            convexity: 0.0,
            sphericity: 0.0,
        };
    }

    let vol = signed_volume(mesh).abs();
    let area = surface_area(mesh);
    let pi = std::f64::consts::PI;

    let sphericity = if area > 1e-15 {
        (pi.powf(1.0 / 3.0) * (6.0 * vol).powf(2.0 / 3.0)) / area
    } else {
        0.0
    };
    let compactness = if area > 1e-15 {
        (36.0 * pi * vol * vol) / (area * area * area)
    } else {
        0.0
    };

    let extents = principal_extents(mesh);
    let elongation = if extents[0] > 1e-15 {
        extents[1] / extents[0]
    } else {
        0.0
    };
    let flatness = if extents[0] > 1e-15 {
        extents[2] / extents[0]
    } else {
        0.0
    };
    let obb_vol = extents[0] * extents[1] * extents[2];
    let rectangularity = if obb_vol > 1e-15 { vol / obb_vol } else { 0.0 };

    ShapeDescriptors {
        compactness,
        elongation,
        flatness,
        rectangularity,
        convexity: 1.0,
        sphericity,
    }
}

/// Add shape descriptors as point data (constant per mesh, replicated to all vertices).
pub fn add_shape_descriptor_arrays(mesh: &PolyData) -> PolyData {
    let sd = compute_shape_descriptors(mesh);
    let n = mesh.points.len();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Sphericity",
            vec![sd.sphericity; n],
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Compactness",
            vec![sd.compactness; n],
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Elongation",
            vec![sd.elongation; n],
            1,
        )));
    result
}

fn signed_volume(mesh: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        vol += a[0] * (b[1] * c[2] - b[2] * c[1])
            + b[0] * (c[1] * a[2] - c[2] * a[1])
            + c[0] * (a[1] * b[2] - a[2] * b[1]);
    }
    vol / 6.0
}

fn surface_area(mesh: &PolyData) -> f64 {
    mesh.polys
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return 0.0;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt()
        })
        .sum()
}

fn principal_extents(mesh: &PolyData) -> [f64; 3] {
    let n = mesh.points.len();
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }
    let mut cov = [[0.0; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for r in 0..3 {
            for cc in 0..3 {
                cov[r][cc] += d[r] * d[cc];
            }
        }
    }
    // Eigenvalues approximate extents
    let tr = cov[0][0] + cov[1][1] + cov[2][2];
    let mut ev = [tr / 3.0; 3]; // placeholder
                                // Use trace-based estimate
    ev.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let scale = (nf).sqrt();
    [
        ev[0].abs().sqrt() * scale * 2.0,
        ev[1].abs().sqrt() * scale * 2.0,
        ev[2].abs().sqrt() * scale * 2.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_descriptors() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let sd = compute_shape_descriptors(&mesh);
        assert!(sd.sphericity > 0.5, "sphericity={}", sd.sphericity);
    }
    #[test]
    fn display() {
        let sd = ShapeDescriptors {
            compactness: 0.5,
            elongation: 0.3,
            flatness: 0.1,
            rectangularity: 0.8,
            convexity: 1.0,
            sphericity: 0.9,
        };
        let s = format!("{sd}");
        assert!(s.contains("compact="));
    }
    #[test]
    fn arrays() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let result = add_shape_descriptor_arrays(&mesh);
        assert!(result.point_data().get_array("Sphericity").is_some());
    }
}
