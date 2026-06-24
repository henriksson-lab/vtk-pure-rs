use crate::data::{AnyDataArray, DataArray, PolyData};

/// Result of connected component counting.
pub struct ComponentCount {
    /// Number of connected components.
    pub count: usize,
}

/// Count the number of connected components in a triangle mesh using union-find.
///
/// Returns the count. If `label` is true, adds a "ComponentId" point data array
/// labeling each vertex with its component index (0-based).
pub fn count_components(input: &PolyData, label: bool) -> (ComponentCount, PolyData) {
    let n: usize = input.points.len();
    if n == 0 {
        return (ComponentCount { count: 0 }, input.clone());
    }

    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    // Union all vertices connected by cells (polys, verts, lines, strips).
    for cells in [&input.polys, &input.verts, &input.lines, &input.strips] {
        for cell in cells.iter() {
            if cell.len() < 2 {
                continue;
            }
            let first: usize = cell[0] as usize;
            for &id in &cell[1..] {
                union(&mut parent, &mut rank, first, id as usize);
            }
        }
    }

    // Collect unique roots.
    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    let count: usize = roots.len();

    let mut pd = input.clone();

    if label {
        // Assign sequential component ids.
        let mut root_to_id: std::collections::HashMap<usize, f64> =
            std::collections::HashMap::new();
        let mut next_id: f64 = 0.0;
        let mut labels: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let root = find(&mut parent, i);
            let id = *root_to_id.entry(root).or_insert_with(|| {
                let v = next_id;
                next_id += 1.0;
                v
            });
            labels.push(id);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "ComponentId",
                labels,
                1,
            )));
    }

    (ComponentCount { count }, pd)
}

fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
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
    use crate::data::{CellArray, Points, PolyData};

    fn make_two_triangles() -> PolyData {
        let mut pd = PolyData::new();
        // Triangle A: vertices 0,1,2
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        // Triangle B: vertices 3,4,5 (disconnected)
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd
    }

    #[test]
    fn two_disconnected_components() {
        let pd = make_two_triangles();
        let (result, _) = count_components(&pd, false);
        assert_eq!(result.count, 2);
    }

    #[test]
    fn single_component() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]); // shares edge with first
        let (result, _) = count_components(&pd, false);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn labels_assigned() {
        let pd = make_two_triangles();
        let (result, labeled) = count_components(&pd, true);
        assert_eq!(result.count, 2);
        let arr = labeled.point_data().get_array("ComponentId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let id_a: f64 = buf[0];
        arr.tuple_as_f64(3, &mut buf);
        let id_b: f64 = buf[0];
        // The two components should have different ids.
        assert!((id_a - id_b).abs() > 0.5);
        // Vertices in the same triangle share an id.
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], id_a);
    }
}
