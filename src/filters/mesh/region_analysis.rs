//! Region analysis: compute area/volume/centroid per labeled region.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Per-region statistics from a cell data label array.
pub fn region_statistics(mesh: &PolyData, label_array: &str) -> Table {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Table::new(),
    };
    let mut buf = [0.0f64];
    let mut regions: std::collections::BTreeMap<i64, (f64, [f64; 3], usize)> =
        std::collections::BTreeMap::new();

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if ci >= arr.num_tuples() {
            break;
        }
        arr.tuple_as_f64(ci, &mut buf);
        let label = buf[0] as i64;
        let area = tri_area(mesh, cell);
        let centroid = cell_centroid(mesh, cell);
        let entry = regions.entry(label).or_insert((0.0, [0.0; 3], 0));
        entry.0 += area;
        for c in 0..3 {
            entry.1[c] += centroid[c] * area;
        }
        entry.2 += 1;
    }

    let mut label_col = Vec::new();
    let mut area_col = Vec::new();
    let mut count_col = Vec::new();
    let mut cx_col = Vec::new();
    let mut cy_col = Vec::new();
    let mut cz_col = Vec::new();

    for (&label, &(area, weighted_centroid, count)) in &regions {
        label_col.push(label as f64);
        area_col.push(area);
        count_col.push(count as f64);
        if area > 1e-15 {
            cx_col.push(weighted_centroid[0] / area);
            cy_col.push(weighted_centroid[1] / area);
            cz_col.push(weighted_centroid[2] / area);
        } else {
            cx_col.push(0.0);
            cy_col.push(0.0);
            cz_col.push(0.0);
        }
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", label_col, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Area", area_col, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CellCount",
            count_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidX",
            cx_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidY",
            cy_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidZ",
            cz_col,
            1,
        )))
}

/// Extract the region with a specific label as a separate PolyData.
pub fn extract_region(mesh: &PolyData, label_array: &str, label: i64) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut buf = [0.0f64];
    let selected: Vec<usize> = (0..all_cells.len())
        .filter(|&ci| {
            if ci < arr.num_tuples() {
                arr.tuple_as_f64(ci, &mut buf);
                buf[0] as i64 == label
            } else {
                false
            }
        })
        .collect();
    extract_cells(mesh, &all_cells, &selected)
}

fn tri_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}
fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut c = [0.0; 3];
    for &pid in cell {
        let p = mesh.points.get(pid as usize);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let k = cell.len() as f64;
    [c[0] / k, c[1] / k, c[2] / k]
}
fn extract_cells(mesh: &PolyData, all_cells: &[Vec<i64>], selected: &[usize]) -> PolyData {
    let mut pts = crate::data::Points::<f64>::new();
    let mut polys = crate::data::CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &ci in selected {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 2.0],
                1,
            )));
        let table = region_statistics(&mesh, "region");
        assert_eq!(table.num_rows(), 2);
    }
    #[test]
    fn extract() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 2.0],
                1,
            )));
        let r = extract_region(&mesh, "region", 1);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
