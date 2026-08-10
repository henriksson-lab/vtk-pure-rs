use crate::data::{AnyDataArray, Points, PolyData};

/// Offset a surface mesh along vertex normals by a given distance.
///
/// If the mesh has a "Normals" point data array, uses those normals.
/// Otherwise, computes face-area-weighted normals automatically.
/// Positive distance moves outward, negative moves inward.
pub fn offset_surface(input: &PolyData, distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Try to use existing normals
    let out_flat = if let Some(arr) = input.point_data().get_array("Normals") {
        if let AnyDataArray::F64(normals) = arr {
            if normals.num_components() == 3 && normals.num_tuples() == n {
                offset_with_f64_normals(input.points.as_flat_slice(), normals.as_slice(), distance)
            } else {
                offset_with_generic_normals(input, arr, distance)
            }
        } else {
            offset_with_generic_normals(input, arr, distance)
        }
    } else {
        // Compute normals from faces
        let normals = compute_vertex_normals(input);
        offset_with_computed_normals(input.points.as_flat_slice(), &normals, distance)
    };

    let mut pd = input.clone();
    pd.points = Points::from_flat_vec(out_flat);
    pd
}

fn offset_with_f64_normals(points: &[f64], normals: &[f64], distance: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(points.len());
    unsafe {
        out.set_len(points.len());
    }
    for i in 0..points.len() {
        out[i] = points[i] + normals[i] * distance;
    }
    out
}

fn offset_with_generic_normals(
    input: &PolyData,
    normals: &AnyDataArray,
    distance: f64,
) -> Vec<f64> {
    let n = input.points.len();
    let points = input.points.as_flat_slice();
    let mut out = Vec::with_capacity(points.len());
    unsafe {
        out.set_len(points.len());
    }
    let mut nbuf = [0.0f64; 3];
    for i in 0..n {
        normals.tuple_as_f64(i, &mut nbuf);
        let b = i * 3;
        out[b] = points[b] + nbuf[0] * distance;
        out[b + 1] = points[b + 1] + nbuf[1] * distance;
        out[b + 2] = points[b + 2] + nbuf[2] * distance;
    }
    out
}

fn offset_with_computed_normals(points: &[f64], normals: &[[f64; 3]], distance: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(points.len());
    unsafe {
        out.set_len(points.len());
    }
    for (i, nm) in normals.iter().enumerate() {
        let b = i * 3;
        out[b] = points[b] + nm[0] * distance;
        out[b + 1] = points[b + 1] + nm[1] * distance;
        out[b + 2] = points[b + 2] + nm[2] * distance;
    }
    out
}

fn compute_vertex_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n = input.points.len();
    let mut normals = vec![[0.0f64; 3]; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for &id in &[cell[0], cell[i], cell[i + 1]] {
                let idx = id as usize;
                normals[idx][0] += fn_[0];
                normals[idx][1] += fn_[1];
                normals[idx][2] += fn_[2];
            }
        }
    }

    for nm in &mut normals {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-15 {
            nm[0] /= len;
            nm[1] /= len;
            nm[2] /= len;
        }
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = offset_surface(&pd, 1.0);
        // All z should be ~1.0 (offset along +Z normal)
        for i in 0..3 {
            let p = result.points.get(i);
            assert!((p[2] - 1.0).abs() < 0.1, "z={} at {}", p[2], i);
        }
    }

    #[test]
    fn negative_offset() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = offset_surface(&pd, -0.5);
        for i in 0..3 {
            let p = result.points.get(i);
            assert!(p[2] < 0.0);
        }
    }

    #[test]
    fn zero_offset() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = offset_surface(&pd, 0.0);
        for i in 0..3 {
            let a = pd.points.get(i);
            let b = result.points.get(i);
            assert!((a[0] - b[0]).abs() < 1e-10);
            assert!((a[1] - b[1]).abs() < 1e-10);
            assert!((a[2] - b[2]).abs() < 1e-10);
        }
    }
}
