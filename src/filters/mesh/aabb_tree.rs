use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

/// Compute the depth of each cell in a BVH-like AABB tree.
///
/// Recursively subdivides cells along the longest axis of their
/// bounding box. Adds "BvhDepth" cell data indicating the subdivision
/// level each cell ends up at.
pub fn bvh_depth(input: &PolyData, max_depth: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();
    if n_cells == 0 {
        return input.clone();
    }

    let mut depths = vec![0.0f64; n_cells];
    let indices: Vec<usize> = (0..n_cells).collect();

    assign_depths(input, &cells, &indices, 0, max_depth, &mut depths);

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BvhDepth", depths, 1,
        )));
    pd
}

fn assign_depths(
    input: &PolyData,
    cells: &[Vec<i64>],
    indices: &[usize],
    depth: usize,
    max_depth: usize,
    depths: &mut [f64],
) {
    if indices.len() <= 1 || depth >= max_depth {
        for &i in indices {
            depths[i] = depth as f64;
        }
        return;
    }

    // Compute bounding box of cell centroids
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    let centroids: Vec<[f64; 3]> = indices
        .iter()
        .map(|&ci| {
            let c = &cells[ci];
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &id in c {
                let p = input.points.get(id as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = c.len() as f64;
            let cent = [cx / n, cy / n, cz / n];
            for k in 0..3 {
                min[k] = min[k].min(cent[k]);
                max[k] = max[k].max(cent[k]);
            }
            cent
        })
        .collect();

    // Split along longest axis
    let dx = max[0] - min[0];
    let dy = max[1] - min[1];
    let dz = max[2] - min[2];
    let axis = if dx >= dy && dx >= dz {
        0
    } else if dy >= dz {
        1
    } else {
        2
    };
    let mid = (min[axis] + max[axis]) * 0.5;

    let mut left = Vec::new();
    let mut right = Vec::new();
    for (ii, &ci) in indices.iter().enumerate() {
        if centroids[ii][axis] < mid {
            left.push(ci);
        } else {
            right.push(ci);
        }
    }

    if left.is_empty() || right.is_empty() {
        for &i in indices {
            depths[i] = depth as f64;
        }
        return;
    }

    assign_depths(input, cells, &left, depth + 1, max_depth, depths);
    assign_depths(input, cells, &right, depth + 1, max_depth, depths);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_basic() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..3 {
            for i in 0..3 {
                let a = (j * 4 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 5]);
                pd.polys.push_cell(&[a, a + 5, a + 4]);
            }
        }

        let result = bvh_depth(&pd, 4);
        let arr = result.cell_data().get_array("BvhDepth").unwrap();
        assert_eq!(arr.num_tuples(), 18);
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] >= 1.0); // at least depth 1
    }

    #[test]
    fn single_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = bvh_depth(&pd, 10);
        let arr = result.cell_data().get_array("BvhDepth").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // single cell = depth 0
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = bvh_depth(&pd, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
