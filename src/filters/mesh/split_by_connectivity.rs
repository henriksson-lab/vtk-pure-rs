use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Split a mesh into separate PolyData objects, one per connected component.
///
/// Uses union-find to identify connected components via shared vertices in
/// polygon cells. Returns a Vec sorted by number of cells (largest first).
pub fn split_by_connectivity(input: &PolyData) -> Vec<PolyData> {
    let n = input.points.len();
    if n == 0 {
        return Vec::new();
    }

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            let Some(ids) = valid_cell_point_ids(cell, n) else {
                continue;
            };
            if ids.len() < 2 {
                continue;
            }
            let first = ids[0];
            for &id in &ids[1..] {
                union(&mut parent, &mut rank, first, id);
            }
        }
    }

    // Group cells by component root.
    let mut component_cells: HashMap<usize, ComponentCells> = HashMap::new();
    group_cells(
        &input.verts,
        n,
        &mut parent,
        &mut component_cells,
        CellKind::Verts,
    );
    group_cells(
        &input.lines,
        n,
        &mut parent,
        &mut component_cells,
        CellKind::Lines,
    );
    group_cells(
        &input.polys,
        n,
        &mut parent,
        &mut component_cells,
        CellKind::Polys,
    );
    group_cells(
        &input.strips,
        n,
        &mut parent,
        &mut component_cells,
        CellKind::Strips,
    );

    // Sort by size (largest first)
    let mut components: Vec<(usize, ComponentCells)> = component_cells.into_iter().collect();
    components.sort_by(|a, b| b.1.num_cells().cmp(&a.1.num_cells()).then(a.0.cmp(&b.0)));

    components
        .into_iter()
        .map(|(_, cells)| build_component(input, &cells))
        .collect()
}

#[derive(Default)]
struct ComponentCells {
    verts: Vec<Vec<i64>>,
    lines: Vec<Vec<i64>>,
    polys: Vec<Vec<i64>>,
    strips: Vec<Vec<i64>>,
}

impl ComponentCells {
    fn num_cells(&self) -> usize {
        self.verts.len() + self.lines.len() + self.polys.len() + self.strips.len()
    }

    fn push(&mut self, kind: CellKind, cell: Vec<i64>) {
        match kind {
            CellKind::Verts => self.verts.push(cell),
            CellKind::Lines => self.lines.push(cell),
            CellKind::Polys => self.polys.push(cell),
            CellKind::Strips => self.strips.push(cell),
        }
    }
}

#[derive(Copy, Clone)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

fn group_cells(
    cells: &CellArray,
    n_points: usize,
    parent: &mut [usize],
    component_cells: &mut HashMap<usize, ComponentCells>,
    kind: CellKind,
) {
    for cell in cells.iter() {
        let Some(ids) = valid_cell_point_ids(cell, n_points) else {
            continue;
        };
        let Some(&first) = ids.first() else {
            continue;
        };
        let root = find(parent, first);
        component_cells
            .entry(root)
            .or_default()
            .push(kind, cell.to_vec());
    }
}

fn build_component(input: &PolyData, cells: &ComponentCells) -> PolyData {
    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut new_points: Points<f64> = Points::new();

    let mut output = PolyData::new();
    output.verts = remap_cell_array(input, &cells.verts, &mut point_map, &mut new_points);
    output.lines = remap_cell_array(input, &cells.lines, &mut point_map, &mut new_points);
    output.polys = remap_cell_array(input, &cells.polys, &mut point_map, &mut new_points);
    output.strips = remap_cell_array(input, &cells.strips, &mut point_map, &mut new_points);
    output.points = new_points;
    output
}

fn remap_cell_array(
    input: &PolyData,
    cells: &[Vec<i64>],
    point_map: &mut HashMap<usize, usize>,
    new_points: &mut Points<f64>,
) -> CellArray {
    let mut remapped_cells = CellArray::new();
    for cell in cells {
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| {
                let old = usize::try_from(id).expect("cell ids were already validated");
                *point_map.entry(old).or_insert_with(|| {
                    let idx = new_points.len();
                    new_points.push(input.points.get(old));
                    idx
                }) as i64
            })
            .collect();
        remapped_cells.push_cell(&remapped);
    }
    remapped_cells
}

fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = find(parent, parent[x]);
    }
    parent[x]
}

fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

fn valid_cell_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_disconnected_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let parts = split_by_connectivity(&pd);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].polys.num_cells(), 1);
        assert_eq!(parts[1].polys.num_cells(), 1);
    }

    #[test]
    fn single_connected_mesh() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let parts = split_by_connectivity(&pd);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].polys.num_cells(), 2);
    }

    #[test]
    fn three_components_sorted_by_size() {
        let pd = PolyData::from_triangles(
            vec![
                // Component A: 2 triangles (4 points sharing edge 1-2)
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                // Component B: 1 triangle
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.5, 1.0, 0.0],
                // Component C: 1 triangle
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [4, 5, 6], [7, 8, 9]],
        );

        let parts = split_by_connectivity(&pd);
        assert_eq!(parts.len(), 3);
        // Largest component first
        assert_eq!(parts[0].polys.num_cells(), 2);
        assert_eq!(parts[1].polys.num_cells(), 1);
        assert_eq!(parts[2].polys.num_cells(), 1);
    }

    #[test]
    fn preserves_non_polygon_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.verts.push_cell(&[2]);
        pd.verts.push_cell(&[3]);

        let parts = split_by_connectivity(&pd);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].lines.num_cells(), 1);
        assert_eq!(parts[1].verts.num_cells(), 1);
        assert_eq!(parts[2].verts.num_cells(), 1);
    }

    #[test]
    fn skips_invalid_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, -1, 1]);
        pd.polys.push_cell(&[0, 99, 1]);

        let parts = split_by_connectivity(&pd);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].lines.num_cells(), 1);
        assert_eq!(parts[0].polys.num_cells(), 0);
    }
}
