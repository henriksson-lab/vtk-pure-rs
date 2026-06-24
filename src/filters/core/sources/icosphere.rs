use std::collections::HashMap;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating an icosphere.
pub struct IcosphereParams {
    pub center: [f64; 3],
    pub radius: f64,
    /// Number of subdivision iterations. 0 = icosahedron, 1 = once subdivided, etc.
    pub subdivisions: usize,
}

impl Default for IcosphereParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            radius: 0.5,
            subdivisions: 2,
        }
    }
}

/// Generate an icosphere by recursively subdividing an icosahedron.
///
/// Each subdivision replaces each triangle with 4 smaller triangles by
/// splitting edges at their midpoints and projecting onto the sphere.
pub fn icosphere(params: &IcosphereParams) -> PolyData {
    let [cx, cy, cz] = params.center;
    let r = params.radius;

    // Start with an icosahedron
    let t = (1.0 + 5.0_f64.sqrt()) / 2.0;

    let mut vertices: Vec<[f64; 3]> = vec![
        normalize([-1.0, t, 0.0]),
        normalize([1.0, t, 0.0]),
        normalize([-1.0, -t, 0.0]),
        normalize([1.0, -t, 0.0]),
        normalize([0.0, -1.0, t]),
        normalize([0.0, 1.0, t]),
        normalize([0.0, -1.0, -t]),
        normalize([0.0, 1.0, -t]),
        normalize([t, 0.0, -1.0]),
        normalize([t, 0.0, 1.0]),
        normalize([-t, 0.0, -1.0]),
        normalize([-t, 0.0, 1.0]),
    ];

    let mut triangles: Vec<[usize; 3]> = vec![
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
        let mut new_triangles = Vec::with_capacity(triangles.len() * 4);
        let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

        for tri in &triangles {
            let a = tri[0];
            let b = tri[1];
            let c = tri[2];

            let ab = get_midpoint(a, b, &mut vertices, &mut midpoint_cache);
            let bc = get_midpoint(b, c, &mut vertices, &mut midpoint_cache);
            let ca = get_midpoint(c, a, &mut vertices, &mut midpoint_cache);

            new_triangles.push([a, ab, ca]);
            new_triangles.push([b, bc, ab]);
            new_triangles.push([c, ca, bc]);
            new_triangles.push([ab, bc, ca]);
        }

        triangles = new_triangles;
    }

    // Build PolyData
    let mut points = Points::new();
    let mut normals_data = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for v in &vertices {
        points.push([cx + r * v[0], cy + r * v[1], cz + r * v[2]]);
        // For a sphere centered at origin, the normal is the normalized position
        normals_data.push_tuple(v);
    }

    for tri in &triangles {
        polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals_data.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

fn get_midpoint(
    a: usize,
    b: usize,
    vertices: &mut Vec<[f64; 3]>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let va = vertices[a];
    let vb = vertices[b];
    let mid = normalize([
        (va[0] + vb[0]) / 2.0,
        (va[1] + vb[1]) / 2.0,
        (va[2] + vb[2]) / 2.0,
    ]);
    let idx = vertices.len();
    vertices.push(mid);
    cache.insert(key, idx);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosahedron() {
        let pd = icosphere(&IcosphereParams {
            subdivisions: 0,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 20);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn one_subdivision() {
        let pd = icosphere(&IcosphereParams {
            subdivisions: 1,
            ..Default::default()
        });
        // 12 original + 30 edge midpoints = 42 vertices
        assert_eq!(pd.points.len(), 42);
        // 20 * 4 = 80 triangles
        assert_eq!(pd.polys.num_cells(), 80);
    }

    #[test]
    fn default_icosphere() {
        let pd = icosphere(&IcosphereParams::default());
        // subdivisions=2: 162 vertices, 320 triangles
        assert_eq!(pd.points.len(), 162);
        assert_eq!(pd.polys.num_cells(), 320);
    }

    #[test]
    fn points_on_sphere() {
        let params = IcosphereParams {
            radius: 2.0,
            subdivisions: 1,
            center: [0.0, 0.0, 0.0],
        };
        let pd = icosphere(&params);
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let dist = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(
                (dist - 2.0).abs() < 1e-10,
                "Point not on sphere: dist={}",
                dist
            );
        }
    }
}
