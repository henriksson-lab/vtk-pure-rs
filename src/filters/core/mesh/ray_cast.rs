use crate::data::PolyData;

/// Result of a ray-mesh intersection test.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Index of the hit cell (triangle).
    pub cell_id: usize,
    /// Hit point position.
    pub point: [f64; 3],
    /// Parametric distance along the ray.
    pub t: f64,
}

/// Cast a ray and find the first intersection with a triangle mesh.
///
/// Ray is defined by `origin` and `direction`. Returns the closest hit.
pub fn ray_cast(input: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dlen < 1e-15 {
        return None;
    }
    let dir = [
        direction[0] / dlen,
        direction[1] / dlen,
        direction[2] / dlen,
    ];

    let mut best: Option<RayHit> = None;

    for (ci, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            if let Some(t) = ray_triangle(origin, dir, v0, v1, v2) {
                if t > 1e-12 {
                    let hit = match &best {
                        Some(b) => t < b.t,
                        None => true,
                    };
                    if hit {
                        best = Some(RayHit {
                            cell_id: ci,
                            point: [
                                origin[0] + t * dir[0],
                                origin[1] + t * dir[1],
                                origin[2] + t * dir[2],
                            ],
                            t,
                        });
                    }
                }
            }
        }
    }
    best
}

/// Cast a ray and find ALL intersections (sorted by distance).
pub fn ray_cast_all(input: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Vec<RayHit> {
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dlen < 1e-15 {
        return vec![];
    }
    let dir = [
        direction[0] / dlen,
        direction[1] / dlen,
        direction[2] / dlen,
    ];

    let mut hits = Vec::new();
    for (ci, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let mut cell_hit: Option<RayHit> = None;
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            if let Some(t) = ray_triangle(origin, dir, v0, v1, v2) {
                if t > 1e-12 {
                    let hit = match &cell_hit {
                        Some(b) => t < b.t,
                        None => true,
                    };
                    if hit {
                        cell_hit = Some(RayHit {
                            cell_id: ci,
                            point: [
                                origin[0] + t * dir[0],
                                origin[1] + t * dir[1],
                                origin[2] + t * dir[2],
                            ],
                            t,
                        });
                    }
                }
            }
        }
        if let Some(hit) = cell_hit {
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    hits
}

fn ray_triangle(o: [f64; 3], d: [f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> Option<f64> {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    Some(f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd
    }

    #[test]
    fn hit_front() {
        let pd = make_quad();
        let hit = ray_cast(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point[2]).abs() < 1e-10);
        assert!((h.t - 1.0).abs() < 1e-10);
    }

    #[test]
    fn miss() {
        let pd = make_quad();
        let hit = ray_cast(&pd, [5.0, 5.0, 1.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_none());
    }

    #[test]
    fn all_hits() {
        let pd = make_quad();
        let hits = ray_cast_all(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert!(!hits.is_empty());
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert!(ray_cast(&pd, [0.0; 3], [0.0, 0.0, -1.0]).is_none());
    }

    #[test]
    fn polygon_fan_internal_edge_counts_as_one_cell_hit() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let hits = ray_cast_all(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_id, 0);
    }
}
