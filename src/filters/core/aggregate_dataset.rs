//! Aggregate multiple datasets into a single combined dataset.
//!
//! Merges multiple PolyData, ImageData, or mixed datasets into
//! a unified output, useful for gathering distributed data.

use crate::data::*;

/// Aggregate multiple PolyData meshes into one.
///
/// All point data arrays present in every mesh are concatenated.
pub fn aggregate_poly_data(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    if meshes.len() == 1 {
        return meshes[0].clone();
    }
    crate::filters::core::append::append(meshes)
}

/// Aggregate PolyData from a MultiBlockDataSet.
pub fn aggregate_multi_block_poly_data(mb: &MultiBlockDataSet) -> PolyData {
    let mut meshes = Vec::new();
    for i in 0..mb.num_blocks() {
        if let Some(Block::PolyData(pd)) = mb.block(i) {
            meshes.push(pd.clone());
        }
    }
    let refs: Vec<&PolyData> = meshes.iter().collect();
    aggregate_poly_data(&refs)
}

/// Aggregate statistics across multiple Tables.
///
/// Computes combined min, max, mean, count for each column that
/// appears in all tables.
pub fn aggregate_table_stats(tables: &[&Table]) -> Table {
    if tables.is_empty() {
        return Table::new();
    }

    // Find common column names
    let first_names: Vec<&str> = tables[0].column_names();
    let common_names: Vec<&str> = first_names
        .iter()
        .filter(|&&name| tables.iter().all(|t| t.column_by_name(name).is_some()))
        .cloned()
        .collect();

    let mut result = Table::new();

    for name in &common_names {
        let mut total_sum = 0.0;
        let mut total_count = 0usize;
        let mut global_min = f64::INFINITY;
        let mut global_max = f64::NEG_INFINITY;

        for table in tables {
            let col = table.column_by_name(name).unwrap();
            let mut buf = [0.0f64];
            for i in 0..col.num_tuples() {
                col.tuple_as_f64(i, &mut buf);
                total_sum += buf[0];
                total_count += 1;
                global_min = global_min.min(buf[0]);
                global_max = global_max.max(buf[0]);
            }
        }

        let mean = if total_count > 0 {
            total_sum / total_count as f64
        } else {
            global_min = 0.0;
            global_max = 0.0;
            0.0
        };

        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("{name}_count"),
            vec![total_count as f64],
            1,
        )));
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("{name}_min"),
            vec![global_min],
            1,
        )));
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("{name}_max"),
            vec![global_max],
            1,
        )));
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("{name}_mean"),
            vec![mean],
            1,
        )));
    }

    result
}

/// Concatenate multiple Tables vertically (row-wise).
pub fn aggregate_tables_vertical(tables: &[&Table]) -> Table {
    if tables.is_empty() {
        return Table::new();
    }
    if tables.len() == 1 {
        return tables[0].clone();
    }

    let names = tables[0].column_names();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); names.len()];

    for table in tables {
        for (ci, name) in names.iter().enumerate() {
            if let Some(col) = table.column_by_name(name) {
                let mut buf = [0.0f64];
                for i in 0..col.num_tuples() {
                    col.tuple_as_f64(i, &mut buf);
                    columns[ci].push(buf[0]);
                }
            }
        }
    }

    let mut result = Table::new();
    for (ci, name) in names.iter().enumerate() {
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &name.to_string(),
            columns[ci].clone(),
            1,
        )));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_two_meshes() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = aggregate_poly_data(&[&a, &b]);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn aggregate_from_multiblock() {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block(
            "a",
            Block::PolyData(PolyData::from_points(vec![[0.0, 0.0, 0.0]])),
        );
        mb.add_block(
            "b",
            Block::PolyData(PolyData::from_points(vec![[1.0, 0.0, 0.0]])),
        );
        let result = aggregate_multi_block_poly_data(&mb);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn table_stats() {
        let t1 = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0],
            1,
        )));
        let t2 = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![3.0, 4.0],
            1,
        )));
        let stats = aggregate_table_stats(&[&t1, &t2]);
        assert!(stats.column_by_name("x_mean").is_some());
        assert_eq!(stats.value_f64(0, "x_mean"), Some(2.5));
    }

    #[test]
    fn vertical_concat() {
        let t1 = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0],
            1,
        )));
        let t2 =
            Table::new().with_column(AnyDataArray::F64(DataArray::from_vec("x", vec![3.0], 1)));
        let result = aggregate_tables_vertical(&[&t1, &t2]);
        assert_eq!(result.num_rows(), 3);
    }

    #[test]
    fn table_stats_empty_columns_are_finite() {
        let t1 = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec("x", vec![], 1)));
        let t2 = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec("x", vec![], 1)));
        let stats = aggregate_table_stats(&[&t1, &t2]);
        assert_eq!(stats.value_f64(0, "x_count"), Some(0.0));
        assert_eq!(stats.value_f64(0, "x_min"), Some(0.0));
        assert_eq!(stats.value_f64(0, "x_max"), Some(0.0));
        assert_eq!(stats.value_f64(0, "x_mean"), Some(0.0));
    }
}
