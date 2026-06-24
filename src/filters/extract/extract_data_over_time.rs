//! Extract data at a specific point over time from a TemporalDataSet.

use crate::data::{AnyDataArray, DataArray, PolyData, Table, TemporalDataSet};

/// Extract scalar values at the closest point to `probe_point` across all time steps.
///
/// Returns a Table with columns "Time" and one column per scalar array.
pub fn extract_data_over_time(temporal: &TemporalDataSet, probe_point: [f64; 3]) -> Table {
    let times = temporal.times();
    if times.is_empty() {
        return Table::new();
    }

    let mut time_col: Vec<f64> = Vec::new();
    let mut data_cols: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();

    for (ti, &t) in times.iter().enumerate() {
        let mesh = match temporal.step(ti) {
            Some(m) => m,
            None => continue,
        };
        if mesh.points.len() == 0 {
            continue;
        }

        // Find closest point
        let closest = find_closest_point(mesh, probe_point);
        time_col.push(t);

        // Extract all scalar arrays at that point
        let pd = mesh.point_data();
        for ai in 0..pd.num_arrays() {
            if let Some(arr) = pd.get_array_by_index(ai) {
                if arr.num_components() != 1 {
                    continue;
                }
                let name = arr.name().to_string();
                let mut buf = [0.0f64];
                arr.tuple_as_f64(closest, &mut buf);
                data_cols.entry(name).or_default().push(buf[0]);
            }
        }
    }

    let n = time_col.len();
    let mut table = Table::new();
    table.add_column(AnyDataArray::F64(DataArray::from_vec("Time", time_col, 1)));

    for (name, mut values) in data_cols {
        // Pad if some time steps didn't have this array
        values.resize(n, f64::NAN);
        table.add_column(AnyDataArray::F64(DataArray::from_vec(&name, values, 1)));
    }

    table
}

/// Extract position of the closest point over time.
pub fn extract_position_over_time(temporal: &TemporalDataSet, probe_point: [f64; 3]) -> Table {
    let times = temporal.times();
    let mut time_col = Vec::new();
    let mut x_col = Vec::new();
    let mut y_col = Vec::new();
    let mut z_col = Vec::new();

    for (ti, &t) in times.iter().enumerate() {
        let mesh = match temporal.step(ti) {
            Some(m) if m.points.len() > 0 => m,
            _ => continue,
        };
        let idx = find_closest_point(mesh, probe_point);
        let p = mesh.points.get(idx);
        time_col.push(t);
        x_col.push(p[0]);
        y_col.push(p[1]);
        z_col.push(p[2]);
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec("Time", time_col, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("X", x_col, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Y", y_col, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Z", z_col, 1)))
}

fn find_closest_point(mesh: &PolyData, target: [f64; 3]) -> usize {
    let mut best = 0;
    let mut best_d2 = f64::MAX;
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        let d2 =
            (p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2) + (p[2] - target[2]).powi(2);
        if d2 < best_d2 {
            best_d2 = d2;
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temporal() -> TemporalDataSet {
        let mut ts = TemporalDataSet::new();
        for i in 0..5 {
            let mut mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
            mesh.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "temp",
                    vec![i as f64 * 10.0, i as f64 * 20.0],
                    1,
                )));
            ts.add_step(i as f64, mesh);
        }
        ts
    }

    #[test]
    fn data_over_time() {
        let ts = make_temporal();
        let table = extract_data_over_time(&ts, [0.0, 0.0, 0.0]);
        assert_eq!(table.num_rows(), 5);
        assert!(table.column_by_name("Time").is_some());
        assert!(table.column_by_name("temp").is_some());
        // At point [0,0,0], temp should be 0, 10, 20, 30, 40
        assert_eq!(table.value_f64(2, "temp"), Some(20.0));
    }

    #[test]
    fn position_over_time() {
        let ts = make_temporal();
        let table = extract_position_over_time(&ts, [0.5, 0.0, 0.0]);
        assert_eq!(table.num_rows(), 5);
        assert!(table.column_by_name("X").is_some());
    }

    #[test]
    fn empty_temporal() {
        let ts = TemporalDataSet::new();
        let table = extract_data_over_time(&ts, [0.0, 0.0, 0.0]);
        assert_eq!(table.num_rows(), 0);
    }
}
