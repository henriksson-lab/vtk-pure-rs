use crate::data::{CellArray, Points, PolyData};

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

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        let first = cell[0] as usize;
        for &id in &cell[1..] {
            union(&mut parent, &mut rank, first, id as usize);
        }
    }

    for cells in [&input.verts, &input.lines, &input.strips] {
        for cell in cells.iter() {
            if cell.len() < 2 {
                continue;
            }
            let first = cell[0] as usize;
            for &id in &cell[1..] {
                union(&mut parent, &mut rank, first, id as usize);
            }
        }
    }

    // Group poly cells by component root
    let mut component_cells: std::collections::HashMap<usize, Vec<Vec<i64>>> =
        std::collections::HashMap::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let root = find(&mut parent, cell[0] as usize);
        component_cells.entry(root).or_default().push(cell.to_vec());
    }

    // Sort by size (largest first)
    let mut components: Vec<(usize, Vec<Vec<i64>>)> = component_cells.into_iter().collect();
    components.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    components
        .into_iter()
        .map(|(_, cells)| build_component(input, &cells))
        .collect()
}

fn build_component(input: &PolyData, cells: &[Vec<i64>]) -> PolyData {
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut new_points: Points<f64> = Points::new();
    let mut polys = CellArray::new();

    for cell in cells {
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| {
                let old = id as usize;
                *point_map.entry(old).or_insert_with(|| {
                    let idx = new_points.len();
                    new_points.push(input.points.get(old));
                    idx
                }) as i64
            })
            .collect();
        polys.push_cell(&remapped);
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = polys;
    output
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
}
