use crate::data::{AnyDataArray, DataArray, PolyData};

/// Approximate ambient occlusion per vertex.
///
/// For each vertex, casts random rays from the vertex along the hemisphere
/// defined by the vertex normal and counts how many intersect the mesh.
/// The occlusion value is the fraction of rays that are occluded.
/// Adds a 1-component "AmbientOcclusion" array to point data.
///
/// Uses a simple hash-based PRNG for reproducibility.
pub fn compute_ambient_occlusion(input: &PolyData, num_rays: usize, max_distance: f64) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute vertex normals by averaging face normals
    let vertex_normals = compute_vertex_normals(input);

    // Collect triangles as index triples
    let triangles = collect_triangles(input);

    let mut ao_values: Vec<f64> = Vec::with_capacity(n);

    for vi in 0..n {
        let origin = input.points.get(vi);
        let normal = &vertex_normals[vi];

        // Build a local coordinate frame from the normal
        let (tangent, bitangent) = build_tangent_frame(normal);

        let mut occluded_count: usize = 0;
        for ray_i in 0..num_rays {
            // Hash-based PRNG: two pseudo-random values in [0, 1)
            let h1: u64 = hash_u64(vi as u64, ray_i as u64 * 2);
            let h2: u64 = hash_u64(vi as u64, ray_i as u64 * 2 + 1);
            let u: f64 = (h1 as f64) / (u64::MAX as f64);
            let v: f64 = (h2 as f64) / (u64::MAX as f64);

            // Cosine-weighted hemisphere sampling
            let phi: f64 = 2.0 * std::f64::consts::PI * u;
            let cos_theta: f64 = (1.0 - v).sqrt();
            let sin_theta: f64 = v.sqrt();

            let dir_x: f64 = sin_theta * phi.cos() * tangent[0]
                + sin_theta * phi.sin() * bitangent[0]
                + cos_theta * normal[0];
            let dir_y: f64 = sin_theta * phi.cos() * tangent[1]
                + sin_theta * phi.sin() * bitangent[1]
                + cos_theta * normal[1];
            let dir_z: f64 = sin_theta * phi.cos() * tangent[2]
                + sin_theta * phi.sin() * bitangent[2]
                + cos_theta * normal[2];

            let dir = [dir_x, dir_y, dir_z];

            // Test ray against all triangles
            if ray_hits_mesh(&origin, &dir, max_distance, vi, &triangles, input) {
                occluded_count += 1;
            }
        }

        let ao: f64 = if num_rays > 0 {
            occluded_count as f64 / num_rays as f64
        } else {
            0.0
        };
        ao_values.push(ao);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AmbientOcclusion",
            ao_values,
            1,
        )));
    pd
}

/// Simple hash combining two u64 values (splitmix64-style).
fn hash_u64(a: u64, b: u64) -> u64 {
    let mut x: u64 = a.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(b);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn compute_vertex_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n: usize = input.points.len();
    let mut normals: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell
            .iter()
            .all(|&idx| idx >= 0 && (idx as usize) < input.points.len())
        {
            continue;
        }
        let a = input.points.get(cell[0] as usize);
        let b = input.points.get(cell[1] as usize);
        let c = input.points.get(cell[2] as usize);

        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_x: f64 = e1[1] * e2[2] - e1[2] * e2[1];
        let fn_y: f64 = e1[2] * e2[0] - e1[0] * e2[2];
        let fn_z: f64 = e1[0] * e2[1] - e1[1] * e2[0];

        for &idx in cell {
            let i: usize = idx as usize;
            normals[i][0] += fn_x;
            normals[i][1] += fn_y;
            normals[i][2] += fn_z;
        }
    }

    // Normalize
    for n in &mut normals {
        let len: f64 = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-20 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            *n = [0.0, 0.0, 1.0];
        }
    }

    normals
}

fn build_tangent_frame(normal: &[f64; 3]) -> ([f64; 3], [f64; 3]) {
    let up: [f64; 3] = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    // tangent = cross(normal, up), then normalize
    let tx: f64 = normal[1] * up[2] - normal[2] * up[1];
    let ty: f64 = normal[2] * up[0] - normal[0] * up[2];
    let tz: f64 = normal[0] * up[1] - normal[1] * up[0];
    let len: f64 = (tx * tx + ty * ty + tz * tz).sqrt();
    let tangent = [tx / len, ty / len, tz / len];

    // bitangent = cross(normal, tangent)
    let bx: f64 = normal[1] * tangent[2] - normal[2] * tangent[1];
    let by: f64 = normal[2] * tangent[0] - normal[0] * tangent[2];
    let bz: f64 = normal[0] * tangent[1] - normal[1] * tangent[0];
    let bitangent = [bx, by, bz];

    (tangent, bitangent)
}

fn collect_triangles(input: &PolyData) -> Vec<[usize; 3]> {
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell
            .iter()
            .all(|&idx| idx >= 0 && (idx as usize) < input.points.len())
        {
            continue;
        }
        // Fan-triangulate polygons
        for i in 1..cell.len() - 1 {
            tris.push([cell[0] as usize, cell[i] as usize, cell[i + 1] as usize]);
        }
    }
    tris
}

/// Moller-Trumbore ray-triangle intersection.
fn ray_triangle_intersect(
    origin: &[f64; 3],
    dir: &[f64; 3],
    v0: &[f64; 3],
    v1: &[f64; 3],
    v2: &[f64; 3],
    max_dist: f64,
) -> bool {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let a: f64 = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];

    if a.abs() < 1e-20 {
        return false;
    }
    let f: f64 = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u: f64 = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v: f64 = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t: f64 = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-6 && t < max_dist
}

fn ray_hits_mesh(
    origin: &[f64; 3],
    dir: &[f64; 3],
    max_distance: f64,
    vertex_idx: usize,
    triangles: &[[usize; 3]],
    input: &PolyData,
) -> bool {
    for tri in triangles {
        // Skip triangles that contain the source vertex to avoid self-intersection
        if tri[0] == vertex_idx || tri[1] == vertex_idx || tri[2] == vertex_idx {
            continue;
        }
        let v0 = input.points.get(tri[0]);
        let v1 = input.points.get(tri[1]);
        let v2 = input.points.get(tri[2]);
        if ray_triangle_intersect(origin, dir, &v0, &v1, &v2, max_distance) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_no_occlusion() {
        // A single flat triangle should have zero occlusion (no geometry to occlude)
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_ambient_occlusion(&pd, 32, 10.0);
        let arr = result.point_data().get_array("AmbientOcclusion").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut val = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            assert!(
                val[0] < 1e-10,
                "expected no occlusion for single triangle, got {}",
                val[0]
            );
        }
    }

    #[test]
    fn ao_values_in_range() {
        // Build a simple box-like shape to get some occlusion
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3], // bottom
                [4, 6, 5],
                [4, 7, 6], // top
                [0, 4, 5],
                [0, 5, 1], // front
                [2, 6, 7],
                [2, 7, 3], // back
            ],
        );
        let result = compute_ambient_occlusion(&pd, 16, 10.0);
        let arr = result.point_data().get_array("AmbientOcclusion").unwrap();
        assert_eq!(arr.num_tuples(), 8);
        let mut val = [0.0f64];
        for i in 0..8 {
            arr.tuple_as_f64(i, &mut val);
            assert!(
                val[0] >= 0.0 && val[0] <= 1.0,
                "AO out of range: {}",
                val[0]
            );
        }
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = compute_ambient_occlusion(&pd, 10, 5.0);
        assert!(result.point_data().get_array("AmbientOcclusion").is_none());
    }
}
