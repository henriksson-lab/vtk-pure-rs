use crate::data::PolyData;

/// Spatial locator for finding cells in a PolyData mesh.
///
/// Builds an axis-aligned bounding box (AABB) tree over the cells of a
/// PolyData for efficient spatial queries: finding the closest cell to a
/// point and finding all cells within a radius.
///
/// Analogous to VTK's `vtkCellLocator`.
pub struct CellLocator {
    /// Search primitives generated from PolyData cells.
    primitives: Vec<Primitive>,
    /// Original cell index for each stored triangle.
    cell_indices: Vec<usize>,
    /// Total number of input PolyData cells, including unsupported/invalid cells.
    total_cells: usize,
    /// Flat BVH nodes.
    nodes: Vec<BvhNode>,
}

#[derive(Clone)]
enum Primitive {
    Point([f64; 3]),
    Segment([[f64; 3]; 2]),
    Triangle([[f64; 3]; 3]),
}

#[derive(Clone)]
struct Aabb {
    min: [f64; 3],
    max: [f64; 3],
}

struct BvhNode {
    aabb: Aabb,
    /// If leaf: start..start+count in the tris/cell_indices arrays.
    /// If internal: left child = idx+1, right child = right_child.
    start: usize,
    count: usize,
    right_child: usize,
}

impl Aabb {
    fn empty() -> Self {
        Self {
            min: [f64::MAX; 3],
            max: [f64::MIN; 3],
        }
    }

    fn expand_point(&mut self, p: [f64; 3]) {
        for k in 0..3 {
            self.min[k] = self.min[k].min(p[k]);
            self.max[k] = self.max[k].max(p[k]);
        }
    }

    fn expand_primitive(&mut self, primitive: &Primitive) {
        match primitive {
            Primitive::Point(p) => self.expand_point(*p),
            Primitive::Segment(segment) => {
                self.expand_point(segment[0]);
                self.expand_point(segment[1]);
            }
            Primitive::Triangle(tri) => {
                self.expand_point(tri[0]);
                self.expand_point(tri[1]);
                self.expand_point(tri[2]);
            }
        }
    }

    fn center(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    fn longest_axis(&self) -> usize {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        if dx >= dy && dx >= dz {
            0
        } else if dy >= dz {
            1
        } else {
            2
        }
    }

    fn dist2_to_point(&self, p: [f64; 3]) -> f64 {
        let mut d2 = 0.0;
        for k in 0..3 {
            if p[k] < self.min[k] {
                let d = self.min[k] - p[k];
                d2 += d * d;
            } else if p[k] > self.max[k] {
                let d = p[k] - self.max[k];
                d2 += d * d;
            }
        }
        d2
    }
}

const MAX_LEAF_SIZE: usize = 4;

impl CellLocator {
    /// Build a cell locator from a PolyData (triangulates polygon cells via fan).
    pub fn build(pd: &PolyData) -> Self {
        let mut primitives = Vec::new();
        let mut cell_indices = Vec::new();
        let total_cells = pd.total_cells();

        for (ci, cell) in pd.verts.iter().enumerate() {
            if cell.len() != 1 {
                continue;
            }
            let Some(p) = point_for_id(pd, cell[0]) else {
                continue;
            };
            primitives.push(Primitive::Point(p));
            cell_indices.push(ci);
        }

        let line_offset = pd.verts.num_cells();
        for (ci, cell) in pd.lines.iter().enumerate() {
            if cell.len() < 2 {
                continue;
            }
            for i in 0..cell.len() - 1 {
                let Some(p0) = point_for_id(pd, cell[i]) else {
                    continue;
                };
                let Some(p1) = point_for_id(pd, cell[i + 1]) else {
                    continue;
                };
                primitives.push(Primitive::Segment([p0, p1]));
                cell_indices.push(line_offset + ci);
            }
        }

        let poly_offset = line_offset + pd.lines.num_cells();
        for (ci, cell) in pd.polys.iter().enumerate() {
            if cell.len() < 3 {
                continue;
            }
            let Some(v0) = point_for_id(pd, cell[0]) else {
                continue;
            };
            for i in 1..cell.len() - 1 {
                let Some(v1) = point_for_id(pd, cell[i]) else {
                    continue;
                };
                let Some(v2) = point_for_id(pd, cell[i + 1]) else {
                    continue;
                };
                primitives.push(Primitive::Triangle([v0, v1, v2]));
                cell_indices.push(poly_offset + ci);
            }
        }

        let strip_offset = poly_offset + pd.polys.num_cells();
        for (ci, cell) in pd.strips.iter().enumerate() {
            if cell.len() < 3 {
                continue;
            }
            for i in 0..cell.len() - 2 {
                let Some(p0) = point_for_id(pd, cell[i]) else {
                    continue;
                };
                let Some(p1) = point_for_id(pd, cell[i + 1]) else {
                    continue;
                };
                let Some(p2) = point_for_id(pd, cell[i + 2]) else {
                    continue;
                };
                let tri = if i % 2 == 0 {
                    [p0, p1, p2]
                } else {
                    [p1, p0, p2]
                };
                primitives.push(Primitive::Triangle(tri));
                cell_indices.push(strip_offset + ci);
            }
        }

        let n = primitives.len();
        let indices: Vec<usize> = (0..n).collect();
        let mut locator = CellLocator {
            primitives,
            cell_indices,
            total_cells,
            nodes: Vec::new(),
        };

        if n > 0 {
            let mut order: Vec<usize> = indices;
            locator.build_node(&mut order, 0, n, 0);

            // Reorder primitives/cell_indices by the final order.
            let old_primitives = locator.primitives.clone();
            let old_ci = locator.cell_indices.clone();
            for (i, &oi) in order.iter().enumerate() {
                locator.primitives[i] = old_primitives[oi].clone();
                locator.cell_indices[i] = old_ci[oi];
            }
        }

        locator
    }

    fn build_node(
        &mut self,
        order: &mut [usize],
        start: usize,
        end: usize,
        _depth: usize,
    ) -> usize {
        let mut aabb = Aabb::empty();
        for &idx in &order[start..end] {
            aabb.expand_primitive(&self.primitives[idx]);
        }

        let count = end - start;
        let node_idx = self.nodes.len();

        if count <= MAX_LEAF_SIZE {
            self.nodes.push(BvhNode {
                aabb,
                start,
                count,
                right_child: 0,
            });
            return node_idx;
        }

        // Split along longest axis at centroid midpoint
        let axis = aabb.longest_axis();
        let mid_val = aabb.center()[axis];

        // Partition
        let mut lo = start;
        let mut hi = end;
        while lo < hi {
            let centroid_axis = primitive_center(&self.primitives[order[lo]])[axis];
            if centroid_axis < mid_val {
                lo += 1;
            } else {
                hi -= 1;
                order.swap(lo, hi);
            }
        }

        // Avoid degenerate splits
        if lo == start || lo == end {
            lo = start + count / 2;
        }

        // Push placeholder node
        self.nodes.push(BvhNode {
            aabb,
            start,
            count: 0, // 0 = internal node
            right_child: 0,
        });

        // Build children
        self.build_node(order, start, lo, _depth + 1);
        let right = self.build_node(order, lo, end, _depth + 1);
        self.nodes[node_idx].right_child = right;

        node_idx
    }

    /// Find the closest cell to a query point.
    ///
    /// Returns `(cell_index, closest_point, squared_distance)` or `None` if empty.
    pub fn find_closest_cell(&self, point: [f64; 3]) -> Option<(usize, [f64; 3], f64)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best_d2 = f64::MAX;
        let mut best_cell = 0;
        let mut best_pt = [0.0; 3];
        self.search_closest(0, point, &mut best_d2, &mut best_cell, &mut best_pt);
        Some((best_cell, best_pt, best_d2))
    }

    fn search_closest(
        &self,
        node_idx: usize,
        point: [f64; 3],
        best_d2: &mut f64,
        best_cell: &mut usize,
        best_pt: &mut [f64; 3],
    ) {
        let node = &self.nodes[node_idx];

        if node.aabb.dist2_to_point(point) >= *best_d2 {
            return;
        }

        if node.count > 0 {
            // Leaf node
            for i in node.start..node.start + node.count {
                let cp = closest_point_on_primitive(point, &self.primitives[i]);
                let d2 = dist2(point, cp);
                if d2 < *best_d2 {
                    *best_d2 = d2;
                    *best_cell = self.cell_indices[i];
                    *best_pt = cp;
                }
            }
            return;
        }

        // Internal node: visit closer child first
        let left = node_idx + 1;
        let right = node.right_child;

        let dl = self.nodes[left].aabb.dist2_to_point(point);
        let dr = self.nodes[right].aabb.dist2_to_point(point);

        if dl < dr {
            self.search_closest(left, point, best_d2, best_cell, best_pt);
            self.search_closest(right, point, best_d2, best_cell, best_pt);
        } else {
            self.search_closest(right, point, best_d2, best_cell, best_pt);
            self.search_closest(left, point, best_d2, best_cell, best_pt);
        }
    }

    /// Find all cells within `radius` distance of a query point.
    ///
    /// Returns `Vec<(cell_index, squared_distance)>`.
    pub fn find_cells_within_radius(&self, point: [f64; 3], radius: f64) -> Vec<(usize, f64)> {
        let mut results = Vec::new();
        if !self.nodes.is_empty() {
            let r2 = radius * radius;
            self.search_radius(0, point, r2, &mut results);
        }
        // Deduplicate by cell index (keep closest)
        results.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        results.dedup_by_key(|x| x.0);
        results
    }

    fn search_radius(
        &self,
        node_idx: usize,
        point: [f64; 3],
        r2: f64,
        results: &mut Vec<(usize, f64)>,
    ) {
        let node = &self.nodes[node_idx];

        if node.aabb.dist2_to_point(point) > r2 {
            return;
        }

        if node.count > 0 {
            for i in node.start..node.start + node.count {
                let cp = closest_point_on_primitive(point, &self.primitives[i]);
                let d2 = dist2(point, cp);
                if d2 <= r2 {
                    results.push((self.cell_indices[i], d2));
                }
            }
            return;
        }

        self.search_radius(node_idx + 1, point, r2, results);
        self.search_radius(node.right_child, point, r2, results);
    }

    /// Number of cells stored.
    pub fn num_cells(&self) -> usize {
        self.total_cells
    }

    /// Number of valid search primitives in the locator.
    pub fn num_primitives(&self) -> usize {
        self.primitives.len()
    }
}

fn primitive_center(primitive: &Primitive) -> [f64; 3] {
    match primitive {
        Primitive::Point(p) => *p,
        Primitive::Segment([a, b]) => scale(add(*a, *b), 0.5),
        Primitive::Triangle([a, b, c]) => scale(add(add(*a, *b), *c), 1.0 / 3.0),
    }
}

fn closest_point_on_primitive(p: [f64; 3], primitive: &Primitive) -> [f64; 3] {
    match primitive {
        Primitive::Point(q) => *q,
        Primitive::Segment(segment) => closest_point_on_segment(p, segment),
        Primitive::Triangle(tri) => closest_point_on_triangle(p, tri),
    }
}

fn closest_point_on_segment(p: [f64; 3], segment: &[[f64; 3]; 2]) -> [f64; 3] {
    let a = segment[0];
    let ab = sub(segment[1], a);
    let denom = dot(ab, ab);
    if denom <= f64::EPSILON {
        return a;
    }
    let t = (dot(sub(p, a), ab) / denom).clamp(0.0, 1.0);
    add(a, scale(ab, t))
}

fn point_for_id(pd: &PolyData, id: i64) -> Option<[f64; 3]> {
    usize::try_from(id)
        .ok()
        .filter(|&idx| idx < pd.points.len())
        .map(|idx| pd.points.get(idx))
}

/// Closest point on a triangle to a query point.
fn closest_point_on_triangle(p: [f64; 3], tri: &[[f64; 3]; 3]) -> [f64; 3] {
    let a = tri[0];
    let b = tri[1];
    let c = tri[2];

    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add(a, scale(ab, v));
    }

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add(a, scale(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add(b, scale(sub(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add(a, add(scale(ab, v), scale(ac, w)))
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
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
    fn closest_cell_on_surface() {
        let pd = make_quad();
        let loc = CellLocator::build(&pd);

        let (_, cp, d2) = loc.find_closest_cell([0.5, 0.5, 0.0]).unwrap();
        assert!(d2 < 1e-10);
        assert!((cp[2]).abs() < 1e-10);
    }

    #[test]
    fn closest_cell_above_surface() {
        let pd = make_quad();
        let loc = CellLocator::build(&pd);

        let (_, cp, d2) = loc.find_closest_cell([0.5, 0.5, 1.0]).unwrap();
        assert!((d2 - 1.0).abs() < 1e-10);
        assert!((cp[2]).abs() < 1e-10);
    }

    #[test]
    fn closest_cell_at_corner() {
        let pd = make_quad();
        let loc = CellLocator::build(&pd);

        let (_, cp, d2) = loc.find_closest_cell([-1.0, -1.0, 0.0]).unwrap();
        // Closest point should be the origin corner
        assert!((d2 - 2.0).abs() < 1e-10);
        assert!((cp[0]).abs() < 1e-10);
        assert!((cp[1]).abs() < 1e-10);
    }

    #[test]
    fn cells_within_radius() {
        let pd = make_quad();
        let loc = CellLocator::build(&pd);

        let results = loc.find_cells_within_radius([0.5, 0.5, 0.0], 0.1);
        // Both triangles should be found (the point is on the shared edge)
        assert!(results.len() >= 1);
    }

    #[test]
    fn cells_within_large_radius() {
        let pd = make_quad();
        let loc = CellLocator::build(&pd);

        let results = loc.find_cells_within_radius([0.5, 0.5, 0.0], 10.0);
        assert_eq!(results.len(), 2); // both cells
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let loc = CellLocator::build(&pd);
        assert!(loc.find_closest_cell([0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn build_skips_invalid_polygon_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);

        let loc = CellLocator::build(&pd);
        assert_eq!(loc.num_cells(), 3);
        assert_eq!(loc.num_primitives(), 1);
        assert!(loc.find_closest_cell([0.25, 0.25, 0.0]).is_some());
    }

    #[test]
    fn locator_includes_vertex_and_line_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[1, 2]);

        let loc = CellLocator::build(&pd);
        assert_eq!(loc.num_cells(), 2);
        assert_eq!(loc.num_primitives(), 2);

        let (cell, point, d2) = loc.find_closest_cell([1.5, 0.5, 0.0]).unwrap();
        assert_eq!(cell, 1);
        assert_eq!(point, [1.5, 0.0, 0.0]);
        assert!((d2 - 0.25).abs() < 1e-10);
    }

    #[test]
    fn locator_includes_triangle_strips_with_polydata_cell_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let loc = CellLocator::build(&pd);
        assert_eq!(loc.num_cells(), 4);
        assert_eq!(loc.num_primitives(), 5);

        let results = loc.find_cells_within_radius([0.75, 0.75, 0.0], 0.01);
        assert!(results.iter().any(|&(cell_id, _)| cell_id == 3));
    }
}
