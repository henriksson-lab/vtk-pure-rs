use crate::data::{AnyDataArray, DataArray, PolyData};
use std::cmp::Ordering;

const LEAF_SIZE: usize = 8;

/// Compute the distance from each point of `source` to the nearest polygon in `target`.
///
/// Adds a "Distance" scalar to `source`'s point data.
pub fn poly_data_distance(source: &PolyData, target: &PolyData) -> PolyData {
    let n_src = source.points.len();
    let locator = TriangleLocator::new(collect_triangles(target));

    if locator.is_empty() {
        return source.clone();
    }

    let mut distances = vec![0.0f64; n_src];
    for i in 0..n_src {
        let p = source.points.get(i);
        distances[i] = locator.nearest_dist2(p).sqrt();
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    pd.point_data_mut().set_active_scalars("Distance");
    pd
}

/// Compute the symmetric Hausdorff-like distance statistics between two surfaces.
///
/// Returns (max_dist_a_to_b, max_dist_b_to_a, mean_a_to_b, mean_b_to_a).
pub fn distance_stats(a: &PolyData, b: &PolyData) -> (f64, f64, f64, f64) {
    let compute = |src: &PolyData, tgt: &PolyData| -> (f64, f64) {
        let n_src = src.points.len();
        let locator = TriangleLocator::new(collect_triangles(tgt));
        if n_src == 0 || locator.is_empty() {
            return (0.0, 0.0);
        }

        let mut max_d = 0.0f64;
        let mut sum_d = 0.0f64;
        for i in 0..n_src {
            let p = src.points.get(i);
            let d = locator.nearest_dist2(p).sqrt();
            max_d = max_d.max(d);
            sum_d += d;
        }
        (max_d, sum_d / n_src as f64)
    };

    let (max_ab, mean_ab) = compute(a, b);
    let (max_ba, mean_ba) = compute(b, a);
    (max_ab, max_ba, mean_ab, mean_ba)
}

fn collect_triangles(poly_data: &PolyData) -> Vec<Triangle> {
    let n_points = poly_data.points.len();
    let mut triangles = Vec::with_capacity(
        poly_data
            .polys
            .iter()
            .map(|cell| cell.len().saturating_sub(2))
            .sum(),
    );

    for cell in &poly_data.polys {
        if cell.len() < 3 || cell.iter().any(|&id| id < 0 || id as usize >= n_points) {
            continue;
        }

        let a = poly_data.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            triangles.push(Triangle::new(
                a,
                poly_data.points.get(cell[i] as usize),
                poly_data.points.get(cell[i + 1] as usize),
            ));
        }
    }

    triangles
}

#[derive(Clone, Copy)]
struct Triangle {
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    min: [f64; 3],
    max: [f64; 3],
    centroid: [f64; 3],
}

impl Triangle {
    fn new(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let min = [
            a[0].min(b[0]).min(c[0]),
            a[1].min(b[1]).min(c[1]),
            a[2].min(b[2]).min(c[2]),
        ];
        let max = [
            a[0].max(b[0]).max(c[0]),
            a[1].max(b[1]).max(c[1]),
            a[2].max(b[2]).max(c[2]),
        ];
        Self {
            a,
            b,
            c,
            min,
            max,
            centroid: [
                (a[0] + b[0] + c[0]) / 3.0,
                (a[1] + b[1] + c[1]) / 3.0,
                (a[2] + b[2] + c[2]) / 3.0,
            ],
        }
    }
}

struct TriangleLocator {
    triangles: Vec<Triangle>,
    nodes: Vec<BvhNode>,
}

impl TriangleLocator {
    fn new(mut triangles: Vec<Triangle>) -> Self {
        let mut locator = Self {
            triangles: Vec::new(),
            nodes: Vec::new(),
        };
        if !triangles.is_empty() {
            locator.triangles.append(&mut triangles);
            locator.build_node(0, locator.triangles.len());
        }
        locator
    }

    fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    fn nearest_dist2(&self, p: [f64; 3]) -> f64 {
        if self.nodes.is_empty() {
            return f64::MAX;
        }
        self.nearest_node_dist2(0, p, f64::MAX)
    }

    fn build_node(&mut self, start: usize, end: usize) -> usize {
        let node_idx = self.nodes.len();
        let (min, max) = triangle_range_bounds(&self.triangles[start..end]);
        self.nodes.push(BvhNode {
            min,
            max,
            start,
            end,
            left: None,
            right: None,
        });

        if end - start > LEAF_SIZE {
            let axis = widest_axis(min, max);
            self.triangles[start..end].sort_by(|a, b| {
                a.centroid[axis]
                    .partial_cmp(&b.centroid[axis])
                    .unwrap_or(Ordering::Equal)
            });
            let mid = start + (end - start) / 2;
            let left = self.build_node(start, mid);
            let right = self.build_node(mid, end);
            self.nodes[node_idx].left = Some(left);
            self.nodes[node_idx].right = Some(right);
        }

        node_idx
    }

    fn nearest_node_dist2(&self, node_idx: usize, p: [f64; 3], mut best: f64) -> f64 {
        let node = &self.nodes[node_idx];
        if point_aabb_dist2(p, node.min, node.max) >= best {
            return best;
        }

        match (node.left, node.right) {
            (Some(left), Some(right)) => {
                let left_node = &self.nodes[left];
                let right_node = &self.nodes[right];
                let left_d2 = point_aabb_dist2(p, left_node.min, left_node.max);
                let right_d2 = point_aabb_dist2(p, right_node.min, right_node.max);
                if left_d2 <= right_d2 {
                    if left_d2 < best {
                        best = self.nearest_node_dist2(left, p, best);
                    }
                    if right_d2 < best {
                        best = self.nearest_node_dist2(right, p, best);
                    }
                } else {
                    if right_d2 < best {
                        best = self.nearest_node_dist2(right, p, best);
                    }
                    if left_d2 < best {
                        best = self.nearest_node_dist2(left, p, best);
                    }
                }
                best
            }
            _ => {
                for tri in &self.triangles[node.start..node.end] {
                    best = best.min(point_triangle_dist2(p, tri.a, tri.b, tri.c));
                }
                best
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BvhNode {
    min: [f64; 3],
    max: [f64; 3],
    start: usize,
    end: usize,
    left: Option<usize>,
    right: Option<usize>,
}

fn triangle_range_bounds(triangles: &[Triangle]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for tri in triangles {
        for axis in 0..3 {
            min[axis] = min[axis].min(tri.min[axis]);
            max[axis] = max[axis].max(tri.max[axis]);
        }
    }
    (min, max)
}

fn widest_axis(min: [f64; 3], max: [f64; 3]) -> usize {
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    if extent[1] > extent[0] && extent[1] >= extent[2] {
        1
    } else if extent[2] > extent[0] && extent[2] > extent[1] {
        2
    } else {
        0
    }
}

fn point_aabb_dist2(p: [f64; 3], min: [f64; 3], max: [f64; 3]) -> f64 {
    let mut d2 = 0.0;
    for axis in 0..3 {
        let d = if p[axis] < min[axis] {
            min[axis] - p[axis]
        } else if p[axis] > max[axis] {
            p[axis] - max[axis]
        } else {
            0.0
        };
        d2 += d * d;
    }
    d2
}

fn point_triangle_dist2(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return dist2(p, a);
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return dist2(p, b);
    }

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return dist2(p, c);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return dist2(p, [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]]);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return dist2(p, [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]]);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return dist2(
            p,
            [
                b[0] + w * (c[0] - b[0]),
                b[1] + w * (c[1] - b[1]),
                b[2] + w * (c[2] - b[2]),
            ],
        );
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    dist2(
        p,
        [
            a[0] + ab[0] * v + ac[0] * w,
            a[1] + ab[1] * v + ac[1] * w,
            a[2] + ab[2] * v + ac[2] * w,
        ],
    )
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_to_self() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = poly_data_distance(&pd, &pd);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn known_distance() {
        let mut src = PolyData::new();
        src.points.push([1.0, 1.0, 5.0]);

        let tgt = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = poly_data_distance(&src, &tgt);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn symmetric_stats() {
        let mut a = PolyData::new();
        a.points.push([1.0, 1.0, 1.0]);
        a.points.push([2.0, 2.0, 1.0]);

        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let (_, _, mean_ab, _mean_ba) = distance_stats(&a, &b);
        assert!((mean_ab - 1.0).abs() < 1e-10);
    }
}
