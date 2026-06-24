//! Split a mesh into connected components.

use crate::data::{CellArray, Points, PolyData};

/// Split mesh into separate PolyData for each connected component.
pub fn split_connected_components(mesh: &PolyData) -> Vec<PolyData> {
    let npts = mesh.points.len();
    if npts == 0 {
        return vec![];
    }

    // Union-Find
    let mut parent: Vec<usize> = (0..npts).collect();
    let mut rank = vec![0u8; npts];

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        let first = cell[0] as usize;
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, first, cell[i] as usize);
        }
    }

    // Group cells by root
    let mut component_cells: std::collections::HashMap<usize, Vec<Vec<i64>>> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let root = find(&mut parent, cell[0] as usize);
        component_cells.entry(root).or_default().push(cell.to_vec());
    }

    // Build separate meshes
    let mut result = Vec::new();
    for (_, cells) in component_cells {
        let mut used = vec![false; npts];
        for cell in &cells {
            for &v in cell {
                used[v as usize] = true;
            }
        }
        let mut pt_map = vec![0usize; npts];
        let mut pts = Points::<f64>::new();
        for i in 0..npts {
            if used[i] {
                pt_map[i] = pts.len();
                pts.push(mesh.points.get(i));
            }
        }
        let mut polys = CellArray::new();
        for cell in &cells {
            let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
            polys.push_cell(&mapped);
        }
        let mut m = PolyData::new();
        m.points = pts;
        m.polys = polys;
        result.push(m);
    }
    result
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
