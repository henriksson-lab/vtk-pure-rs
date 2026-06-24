//! CSV/TSV reader and writer for vtk-rs Tables and PolyData.
//!
//! Provides flexible delimiter support (comma, tab, semicolon, pipe)
//! and handles quoted fields, headers, and type inference.

use crate::data::{AnyDataArray, DataArray, Points, PolyData, Table};
use std::io::{BufRead, Write};

/// CSV delimiter type.
#[derive(Debug, Clone, Copy)]
pub enum Delimiter {
    Comma,
    Tab,
    Semicolon,
    Pipe,
    Custom(char),
}

impl Delimiter {
    fn char(&self) -> char {
        match self {
            Self::Comma => ',',
            Self::Tab => '\t',
            Self::Semicolon => ';',
            Self::Pipe => '|',
            Self::Custom(c) => *c,
        }
    }
}

/// Read a CSV/TSV file into a Table.
pub fn read_csv<R: BufRead>(reader: R, delimiter: Delimiter) -> Result<Table, String> {
    let delim = delimiter.char();
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or("empty CSV")?
        .map_err(|e| e.to_string())?;
    let col_names: Vec<String> = split_csv_line(&header, delim);
    let ncols = col_names.len();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ncols];

    for line_result in lines {
        let line = line_result.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let values = split_csv_line(trimmed, delim);
        for (i, val) in values.iter().enumerate().take(ncols) {
            columns[i].push(val.parse().unwrap_or(f64::NAN));
        }
    }

    let mut table = Table::new();
    for (i, name) in col_names.iter().enumerate() {
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            name.trim(),
            columns[i].clone(),
            1,
        )));
    }
    Ok(table)
}

/// Write a Table as CSV/TSV.
pub fn write_csv<W: Write>(
    writer: &mut W,
    table: &Table,
    delimiter: Delimiter,
) -> std::io::Result<()> {
    let delim = delimiter.char();
    let names: Vec<&str> = table.columns().iter().map(|c| c.name()).collect();
    writeln!(writer, "{}", names.join(&delim.to_string()))?;

    for row in 0..table.num_rows() {
        let mut vals = Vec::with_capacity(table.num_columns());
        for col in table.columns() {
            let nc = col.num_components();
            let mut buf = vec![0.0f64; nc];
            col.tuple_as_f64(row, &mut buf);
            if nc == 1 {
                vals.push(format_f64(buf[0]));
            } else {
                let parts: Vec<String> = buf.iter().map(|v| format_f64(*v)).collect();
                vals.push(parts.join(";"));
            }
        }
        writeln!(writer, "{}", vals.join(&delim.to_string()))?;
    }
    Ok(())
}

/// Read CSV as PolyData points (expects x, y, z columns).
pub fn read_csv_as_points<R: BufRead>(reader: R, delimiter: Delimiter) -> Result<PolyData, String> {
    let table = read_csv(reader, delimiter)?;
    let n = table.num_rows();
    if n == 0 {
        return Ok(PolyData::new());
    }

    // Find x, y, z columns (case-insensitive)
    let names = table.column_names();
    let x_idx = names.iter().position(|n| {
        let l = n.to_lowercase();
        l == "x" || l == "lon" || l == "longitude"
    });
    let y_idx = names.iter().position(|n| {
        let l = n.to_lowercase();
        l == "y" || l == "lat" || l == "latitude"
    });
    let z_idx = names.iter().position(|n| {
        let l = n.to_lowercase();
        l == "z" || l == "alt" || l == "altitude" || l == "elevation"
    });

    let mut points = Points::<f64>::new();
    for row in 0..n {
        let x = x_idx
            .and_then(|i| table.value_f64(row, names[i]))
            .unwrap_or(0.0);
        let y = y_idx
            .and_then(|i| table.value_f64(row, names[i]))
            .unwrap_or(0.0);
        let z = z_idx
            .and_then(|i| table.value_f64(row, names[i]))
            .unwrap_or(0.0);
        points.push([x, y, z]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;

    // Add remaining columns as point data
    for (ci, name) in names.iter().enumerate() {
        if Some(ci) == x_idx || Some(ci) == y_idx || Some(ci) == z_idx {
            continue;
        }
        if let Some(col) = table.column(ci) {
            let mut data = Vec::with_capacity(n);
            let mut buf = [0.0f64];
            for row in 0..n {
                col.tuple_as_f64(row, &mut buf);
                data.push(buf[0]);
            }
            mesh.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    &name.to_string(),
                    data,
                    1,
                )));
        }
    }

    Ok(mesh)
}

/// Read CSV from file path.
pub fn read_csv_file(path: &std::path::Path) -> Result<Table, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let delim = if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
        Delimiter::Tab
    } else {
        Delimiter::Comma
    };
    read_csv(std::io::BufReader::new(file), delim)
}

/// Write CSV to file path.
pub fn write_csv_file(path: &std::path::Path, table: &Table) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let delim = if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
        Delimiter::Tab
    } else {
        Delimiter::Comma
    };
    write_csv(&mut std::io::BufWriter::new(file), table, delim).map_err(|e| e.to_string())
}

fn split_csv_line(line: &str, delim: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch == delim && !in_quotes {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn format_f64(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_simple_csv() {
        let data = "x,y,z\n1,2,3\n4,5,6\n";
        let table = read_csv(data.as_bytes(), Delimiter::Comma).unwrap();
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.num_columns(), 3);
        assert_eq!(table.value_f64(0, "x"), Some(1.0));
    }

    #[test]
    fn read_tsv() {
        let data = "a\tb\n1\t2\n3\t4\n";
        let table = read_csv(data.as_bytes(), Delimiter::Tab).unwrap();
        assert_eq!(table.num_rows(), 2);
    }

    #[test]
    fn roundtrip() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![3.0, 4.0],
                1,
            )));
        let mut buf = Vec::new();
        write_csv(&mut buf, &table, Delimiter::Comma).unwrap();
        let loaded = read_csv(&buf[..], Delimiter::Comma).unwrap();
        assert_eq!(loaded.num_rows(), 2);
        assert_eq!(loaded.value_f64(0, "x"), Some(1.0));
    }

    #[test]
    fn csv_as_points() {
        let data = "x,y,z,temp\n1,2,3,100\n4,5,6,200\n";
        let mesh = read_csv_as_points(data.as_bytes(), Delimiter::Comma).unwrap();
        assert_eq!(mesh.points.len(), 2);
        assert!(mesh.point_data().get_array("temp").is_some());
    }

    #[test]
    fn quoted_fields() {
        let data = "name,val\n\"hello, world\",42\n";
        let table = read_csv(data.as_bytes(), Delimiter::Comma).unwrap();
        assert_eq!(table.num_rows(), 1);
    }

    #[test]
    fn skip_comments() {
        let data = "x\n#comment\n1\n2\n";
        let table = read_csv(data.as_bytes(), Delimiter::Comma).unwrap();
        assert_eq!(table.num_rows(), 2);
    }
}
