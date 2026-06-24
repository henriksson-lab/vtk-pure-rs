//! Surface area computation per cell data region.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Compute total surface area grouped by a cell data label.
pub fn surface_area_by_label(mesh: &PolyData, label_array: &str) -> Table {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Table::new(),
    };
    let mut buf = [0.0f64];
    let mut areas: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if ci >= arr.num_tuples() {
            break;
        }
        arr.tuple_as_f64(ci, &mut buf);
        let label = buf[0] as i64;
        let area = tri_area(mesh, cell);
        *areas.entry(label).or_insert(0.0) += area;
    }

    let labels: Vec<f64> = areas.keys().map(|&k| k as f64).collect();
    let area_vals: Vec<f64> = areas.values().cloned().collect();
    let total: f64 = area_vals.iter().sum();
    let fractions: Vec<f64> = area_vals
        .iter()
        .map(|&a| if total > 1e-15 { a / total } else { 0.0 })
        .collect();

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            label_array,
            labels,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Area", area_vals, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "AreaFraction",
            fractions,
            1,
        )))
}

/// Compute area-weighted average of a point data scalar per label.
pub fn area_weighted_average_by_label(
    mesh: &PolyData,
    label_array: &str,
    value_array: &str,
) -> Table {
    let lab_arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Table::new(),
    };
    let val_arr = match mesh.point_data().get_array(value_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Table::new(),
    };
    let mut lb = [0.0f64];
    let mut vb = [0.0f64];
    let mut weighted_sums: std::collections::BTreeMap<i64, (f64, f64)> =
        std::collections::BTreeMap::new();

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if ci >= lab_arr.num_tuples() {
            break;
        }
        lab_arr.tuple_as_f64(ci, &mut lb);
        let label = lb[0] as i64;
        let area = tri_area(mesh, cell);
        let avg_val: f64 = cell
            .iter()
            .map(|&pid| {
                val_arr.tuple_as_f64(pid as usize, &mut vb);
                vb[0]
            })
            .sum::<f64>()
            / cell.len() as f64;
        let entry = weighted_sums.entry(label).or_insert((0.0, 0.0));
        entry.0 += avg_val * area;
        entry.1 += area;
    }

    let labels: Vec<f64> = weighted_sums.keys().map(|&k| k as f64).collect();
    let averages: Vec<f64> = weighted_sums
        .values()
        .map(|&(wsum, atot)| if atot > 1e-15 { wsum / atot } else { 0.0 })
        .collect();

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            label_array,
            labels,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            value_array,
            averages,
            1,
        )))
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn by_label() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "r",
                vec![1.0, 2.0],
                1,
            )));
        let table = surface_area_by_label(&mesh, "r");
        assert_eq!(table.num_rows(), 2);
        assert!(table.column_by_name("AreaFraction").is_some());
    }
    #[test]
    fn weighted_avg() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("r", vec![1.0], 1)));
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0],
                1,
            )));
        let table = area_weighted_average_by_label(&mesh, "r", "v");
        assert_eq!(table.num_rows(), 1);
    }
}
