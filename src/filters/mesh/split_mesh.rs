//! Split a mesh into connected components.

use crate::data::{CellArray, Points, PolyData};
use std::collections::BTreeMap;

/// Split mesh into separate PolyData for each connected component.
pub fn split_connected_components(mesh: &PolyData) -> Vec<PolyData> {
    let npts = mesh.points.len();
    if npts == 0 {
        return vec![];
    }

    // Union-Find
    let mut parent: Vec<usize> = (0..npts).collect();
    let mut rank = vec![0u8; npts];

    union_cell_array(&mesh.verts, npts, &mut parent, &mut rank);
    union_cell_array(&mesh.lines, npts, &mut parent, &mut rank);
    union_cell_array(&mesh.polys, npts, &mut parent, &mut rank);
    union_cell_array(&mesh.strips, npts, &mut parent, &mut rank);

    // Group cells by root in deterministic component order.
    let mut component_cells: BTreeMap<usize, ComponentCells> = BTreeMap::new();
    group_cell_array(
        &mesh.verts,
        npts,
        &mut parent,
        &mut component_cells,
        CellKind::Verts,
    );
    group_cell_array(
        &mesh.lines,
        npts,
        &mut parent,
        &mut component_cells,
        CellKind::Lines,
    );
    group_cell_array(
        &mesh.polys,
        npts,
        &mut parent,
        &mut component_cells,
        CellKind::Polys,
    );
    group_cell_array(
        &mesh.strips,
        npts,
        &mut parent,
        &mut component_cells,
        CellKind::Strips,
    );

    // Build separate meshes
    let mut result = Vec::new();
    for (_, cells) in component_cells {
        let mut used = vec![false; npts];
        mark_used(&cells.verts, &mut used);
        mark_used(&cells.lines, &mut used);
        mark_used(&cells.polys, &mut used);
        mark_used(&cells.strips, &mut used);

        let mut pt_map = vec![0usize; npts];
        let mut pts = Points::<f64>::new();
        for i in 0..npts {
            if used[i] {
                pt_map[i] = pts.len();
                pts.push(mesh.points.get(i));
            }
        }
        let mut m = PolyData::new();
        m.points = pts;
        m.verts = remap_cells(&cells.verts, &pt_map);
        m.lines = remap_cells(&cells.lines, &pt_map);
        m.polys = remap_cells(&cells.polys, &pt_map);
        m.strips = remap_cells(&cells.strips, &pt_map);
        result.push(m);
    }
    result
}

#[derive(Clone, Copy)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

#[derive(Default)]
struct ComponentCells {
    verts: Vec<Vec<i64>>,
    lines: Vec<Vec<i64>>,
    polys: Vec<Vec<i64>>,
    strips: Vec<Vec<i64>>,
}

fn union_cell_array(cells: &CellArray, npts: usize, parent: &mut [usize], rank: &mut [u8]) {
    for cell in cells.iter() {
        if cell.len() < 2 || !cell.iter().all(|&id| id >= 0 && (id as usize) < npts) {
            continue;
        }
        let first = cell[0] as usize;
        for &id in &cell[1..] {
            union(parent, rank, first, id as usize);
        }
    }
}

fn group_cell_array(
    cells: &CellArray,
    npts: usize,
    parent: &mut [usize],
    component_cells: &mut BTreeMap<usize, ComponentCells>,
    kind: CellKind,
) {
    for cell in cells.iter() {
        if cell.is_empty() || !cell.iter().all(|&id| id >= 0 && (id as usize) < npts) {
            continue;
        }
        let root = find(parent, cell[0] as usize);
        let component = component_cells.entry(root).or_default();
        match kind {
            CellKind::Verts => component.verts.push(cell.to_vec()),
            CellKind::Lines => component.lines.push(cell.to_vec()),
            CellKind::Polys => component.polys.push(cell.to_vec()),
            CellKind::Strips => component.strips.push(cell.to_vec()),
        }
    }
}

fn mark_used(cells: &[Vec<i64>], used: &mut [bool]) {
    for cell in cells {
        for &v in cell {
            used[v as usize] = true;
        }
    }
}

fn remap_cells(cells: &[Vec<i64>], pt_map: &[usize]) -> CellArray {
    let mut mapped_cells = CellArray::new();
    for cell in cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        mapped_cells.push_cell(&mapped);
    }
    mapped_cells
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn union(parent: &mut [usize], rank: &mut [u8], a: usize, b: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_two_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = split_connected_components(&mesh);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].polys.num_cells(), 1);
        assert_eq!(parts[1].polys.num_cells(), 1);
    }
    #[test]
    fn test_single() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let parts = split_connected_components(&mesh);
        assert_eq!(parts.len(), 1);
    }
}
