use crate::data::PolyData;

/// Displace mesh vertices along their normals by a pseudo-random amount.
///
/// Uses a simple hash-based PRNG for reproducible results. Each vertex is
/// displaced by `amplitude * hash(seed, vertex_index)` along its normal,
/// where hash returns a value in [-1, 1].
///
/// Normals must already be present in point data as "Normals" (3-component).
/// If no normals are found, falls back to displacing along the vertex direction
/// from the centroid.
pub fn noise_displacement(input: &PolyData, amplitude: f64, seed: u64) -> PolyData {
    let n: usize = input.points.len();
    let mut output = input.clone();

    if n == 0 {
        return output;
    }

    // Try to get normals from point data
    let normals_arr = input.point_data().get_array("Normals");
    let has_normals: bool = normals_arr.is_some() && normals_arr.unwrap().num_components() == 3;

    // If no normals, compute centroid for fallback direction
    let mut centroid = [0.0f64; 3];
    if !has_normals {
        for i in 0..n {
            let p = input.points.get(i);
            centroid[0] += p[0];
            centroid[1] += p[1];
            centroid[2] += p[2];
        }
        let inv_n: f64 = 1.0 / n as f64;
        centroid[0] *= inv_n;
        centroid[1] *= inv_n;
        centroid[2] *= inv_n;
    }

    for i in 0..n {
        let p = input.points.get(i);
        let noise: f64 = amplitude * hash_to_f64(seed, i as u64);

        let nx: f64;
        let ny: f64;
        let nz: f64;

        if has_normals {
            let arr = normals_arr.unwrap();
            let mut buf = [0.0f64; 3];
            arr.tuple_as_f64(i, &mut buf);
            nx = buf[0];
            ny = buf[1];
            nz = buf[2];
        } else {
            // Fallback: direction from centroid
            let dx: f64 = p[0] - centroid[0];
            let dy: f64 = p[1] - centroid[1];
            let dz: f64 = p[2] - centroid[2];
            let len: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
            if len > 1e-20 {
                nx = dx / len;
                ny = dy / len;
                nz = dz / len;
            } else {
                nx = 0.0;
                ny = 0.0;
                nz = 1.0;
            }
        }

        let new_p: [f64; 3] = [p[0] + noise * nx, p[1] + noise * ny, p[2] + noise * nz];
        output.points.set(i, new_p);
    }

    output
}

/// Simple hash-based pseudo-random number generator.
/// Returns a value in [-1, 1] for a given seed and index.
fn hash_to_f64(seed: u64, index: u64) -> f64 {
    // Splitmix64-style hash
    let mut z: u64 = seed.wrapping_add(index.wrapping_mul(0x9E3779B97F4A7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z = z ^ (z >> 31);
    // Map to [-1, 1]
    let normalized: f64 = (z as f64) / (u64::MAX as f64);
    normalized * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn displacement_with_normals() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        // Add normals pointing in +Z
        let normals_data: Vec<f64> = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                normals_data,
                3,
            )));

        let result = noise_displacement(&pd, 0.5, 42);
        // Points should only change in Z since normals are +Z
        for i in 0..3 {
            let orig = pd.points.get(i);
            let disp = result.points.get(i);
            assert!((disp[0] - orig[0]).abs() < 1e-10, "x should not change");
            assert!((disp[1] - orig[1]).abs() < 1e-10, "y should not change");
            // z should change by some amount <= amplitude
            assert!((disp[2] - orig[2]).abs() <= 0.5 + 1e-10);
        }
    }

    #[test]
    fn reproducible_with_same_seed() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);

        let r1 = noise_displacement(&pd, 1.0, 123);
        let r2 = noise_displacement(&pd, 1.0, 123);

        for i in 0..pd.points.len() {
            let p1 = r1.points.get(i);
            let p2 = r2.points.get(i);
            assert_eq!(p1, p2, "same seed should give same result");
        }
    }

    #[test]
    fn empty_mesh_unchanged() {
        let pd = PolyData::new();
        let result = noise_displacement(&pd, 1.0, 0);
        assert_eq!(result.points.len(), 0);
    }
}
