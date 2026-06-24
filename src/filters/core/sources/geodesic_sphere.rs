use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Parameters for generating a geodesic sphere (icosphere).
pub struct GeodesicSphereParams {
    /// Radius. Default: 1.0
    pub radius: f64,
    /// Number of subdivision levels (0 = icosahedron, 1 = 42 verts, 2 = 162, ...). Default: 2
    pub subdivisions: usize,
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for GeodesicSphereParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            subdivisions: 2,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a geodesic sphere (icosphere) by subdividing an icosahedron.
///
/// Produces a near-uniform triangulation of the sphere, unlike UV spheres
/// which have pole singularities.
pub fn geodesic_sphere(params: &GeodesicSphereParams) -> PolyData {
    let r = params.radius;

    // Start with icosahedron vertices
    let t = (1.0 + 5.0_f64.sqrt()) * 0.5;
    let len = (1.0 + t * t).sqrt();
    let a = 1.0 / len;
    let b = t / len;

    let mut verts: Vec<[f64; 3]> = vec![
        [-a, b, 0.0],
        [a, b, 0.0],
        [-a, -b, 0.0],
        [a, -b, 0.0],
        [0.0, -a, b],
        [0.0, a, b],
        [0.0, -a, -b],
        [0.0, a, -b],
        [b, 0.0, -a],
        [b, 0.0, a],
        [-b, 0.0, -a],
        [-b, 0.0, a],
    ];

    let mut tris: Vec<[usize; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    // Subdivide
    for _ in 0..params.subdivisions {
        let mut new_tris = Vec::new();
        let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

        for &[a, b, c] in &tris {
            let ab = get_midpoint_idx(&mut verts, &mut midpoint_cache, a, b);
            let bc = get_midpoint_idx(&mut verts, &mut midpoint_cache, b, c);
            let ca = get_midpoint_idx(&mut verts, &mut midpoint_cache, c, a);
            new_tris.push([a, ab, ca]);
            new_tris.push([b, bc, ab]);
            new_tris.push([c, ca, bc]);
            new_tris.push([ab, bc, ca]);
        }
        tris = new_tris;
    }

    // Build output
    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for v in &verts {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let nx = v[0] / len;
        let ny = v[1] / len;
        let nz = v[2] / len;
        points.push([
            params.center[0] + nx * r,
            params.center[1] + ny * r,
            params.center[2] + nz * r,
        ]);
        normals.push_tuple(&[nx, ny, nz]);
    }

    for &[a, b, c] in &tris {
        polys.push_cell(&[a as i64, b as i64, c as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(AnyDataArray::F64(normals));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

fn get_midpoint_idx(
    verts: &mut Vec<[f64; 3]>,
    cache: &mut HashMap<(usize, usize), usize>,
    a: usize,
    b: usize,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = [
        (verts[a][0] + verts[b][0]) * 0.5,
        (verts[a][1] + verts[b][1]) * 0.5,
        (verts[a][2] + verts[b][2]) * 0.5,
    ];
    // Project onto unit sphere
    let len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
    let idx = verts.len();
    verts.push([mid[0] / len, mid[1] / len, mid[2] / len]);
    cache.insert(key, idx);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosahedron() {
        let pd = geodesic_sphere(&GeodesicSphereParams {
            subdivisions: 0,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 20);
    }

    #[test]
    fn subdivision_1() {
        let pd = geodesic_sphere(&GeodesicSphereParams {
            subdivisions: 1,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 42);
        assert_eq!(pd.polys.num_cells(), 80);
    }

    #[test]
    fn subdivision_2() {
        let pd = geodesic_sphere(&GeodesicSphereParams {
            subdivisions: 2,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 162);
        assert_eq!(pd.polys.num_cells(), 320);
    }

    #[test]
    fn points_on_sphere() {
        let pd = geodesic_sphere(&GeodesicSphereParams {
            radius: 2.0,
            subdivisions: 1,
            center: [0.0, 0.0, 0.0],
            ..Default::default()
        });
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 2.0).abs() < 1e-10, "r={} at point {}", r, i);
        }
    }

    #[test]
    fn has_normals() {
        let pd = geodesic_sphere(&GeodesicSphereParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
    }
}
