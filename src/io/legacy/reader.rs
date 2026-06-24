use std::io::BufRead;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::{ScalarType, VtkError};

/// Reader for VTK legacy format (.vtk) files.
pub struct LegacyReader;

impl LegacyReader {
    /// Read a PolyData from a VTK legacy file.
    pub fn read_poly_data(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_poly_data_from(reader)
    }

    /// Read a PolyData from a buffered reader.
    pub fn read_poly_data_from<R: BufRead>(reader: R) -> Result<PolyData, VtkError> {
        let mut parser = Parser::new(reader);
        parser.parse_poly_data()
    }
}

struct Parser<R: BufRead> {
    reader: R,
    is_binary: bool,
}

impl<R: BufRead> Parser<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            is_binary: false,
        }
    }

    fn parse_poly_data(&mut self) -> Result<PolyData, VtkError> {
        // Parse header
        self.parse_header()?;

        let mut pd = PolyData::new();

        // Parse sections until EOF
        while let Ok(line) = self.read_nonempty_line() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            match tokens[0].to_uppercase().as_str() {
                "POINTS" => {
                    pd.points = self.parse_points(&tokens)?;
                }
                "VERTICES" => {
                    pd.verts = self.parse_cells(&tokens)?;
                }
                "LINES" => {
                    pd.lines = self.parse_cells(&tokens)?;
                }
                "POLYGONS" => {
                    pd.polys = self.parse_cells(&tokens)?;
                }
                "TRIANGLE_STRIPS" => {
                    pd.strips = self.parse_cells(&tokens)?;
                }
                "POINT_DATA" => {
                    let n: usize = parse_token(tokens.get(1), "POINT_DATA count")?;
                    self.parse_data_attributes(pd.point_data_mut(), n)?;
                }
                "CELL_DATA" => {
                    let n: usize = parse_token(tokens.get(1), "CELL_DATA count")?;
                    self.parse_data_attributes(pd.cell_data_mut(), n)?;
                }
                _ => {
                    // Skip unknown sections
                }
            }
        }

        Ok(pd)
    }

    fn parse_header(&mut self) -> Result<(), VtkError> {
        // Line 1: version
        let version_line = self.read_line()?;
        if !version_line.starts_with("# vtk DataFile Version") {
            return Err(VtkError::Parse(format!(
                "expected VTK header, got: {}",
                version_line
            )));
        }

        // Line 2: description (skip)
        let _description = self.read_line()?;

        // Line 3: ASCII or BINARY
        let file_type = self.read_line()?;
        match file_type.trim().to_uppercase().as_str() {
            "ASCII" => self.is_binary = false,
            "BINARY" => self.is_binary = true,
            other => {
                return Err(VtkError::Parse(format!(
                    "expected ASCII or BINARY, got: {}",
                    other
                )))
            }
        }

        // Line 4: DATASET type
        let dataset_line = self.read_line()?;
        let tokens: Vec<&str> = dataset_line.split_whitespace().collect();
        if tokens.len() < 2 || tokens[0].to_uppercase() != "DATASET" {
            return Err(VtkError::Parse(format!(
                "expected DATASET line, got: {}",
                dataset_line
            )));
        }
        if tokens[1].to_uppercase() != "POLYDATA" {
            return Err(VtkError::Unsupported(format!(
                "only POLYDATA supported, got: {}",
                tokens[1]
            )));
        }

        Ok(())
    }

    fn parse_points(&mut self, header_tokens: &[&str]) -> Result<Points<f64>, VtkError> {
        // POINTS <n> <type>
        let n: usize = parse_token(header_tokens.get(1), "POINTS count")?;
        let type_name = header_tokens
            .get(2)
            .ok_or_else(|| VtkError::Parse("missing POINTS type".into()))?;
        let scalar_type = ScalarType::from_vtk_name(type_name)
            .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;

        if self.is_binary {
            self.parse_points_binary(n, scalar_type)
        } else {
            self.parse_points_ascii(n)
        }
    }

    fn parse_points_ascii(&mut self, n: usize) -> Result<Points<f64>, VtkError> {
        let mut pts = Points::new();
        let mut values = Vec::with_capacity(n * 3);

        // Read all float values (may be spread across multiple lines)
        while values.len() < n * 3 {
            let line = self.read_nonempty_line()?;
            for token in line.split_whitespace() {
                let v: f64 = token
                    .parse()
                    .map_err(|_| VtkError::Parse(format!("invalid float: {}", token)))?;
                values.push(v);
            }
        }

        for i in 0..n {
            pts.push([values[i * 3], values[i * 3 + 1], values[i * 3 + 2]]);
        }
        Ok(pts)
    }

    fn parse_points_binary(
        &mut self,
        n: usize,
        scalar_type: ScalarType,
    ) -> Result<Points<f64>, VtkError> {
        let mut pts = Points::new();
        match scalar_type {
            ScalarType::F32 => {
                let mut buf = vec![0u8; n * 3 * 4];
                self.reader.read_exact(&mut buf)?;
                for i in 0..n {
                    let base = i * 3 * 4;
                    let x = f32::from_be_bytes(buf[base..base + 4].try_into().unwrap()) as f64;
                    let y = f32::from_be_bytes(buf[base + 4..base + 8].try_into().unwrap()) as f64;
                    let z = f32::from_be_bytes(buf[base + 8..base + 12].try_into().unwrap()) as f64;
                    pts.push([x, y, z]);
                }
            }
            ScalarType::F64 => {
                let mut buf = vec![0u8; n * 3 * 8];
                self.reader.read_exact(&mut buf)?;
                for i in 0..n {
                    let base = i * 3 * 8;
                    let x = f64::from_be_bytes(buf[base..base + 8].try_into().unwrap());
                    let y = f64::from_be_bytes(buf[base + 8..base + 16].try_into().unwrap());
                    let z = f64::from_be_bytes(buf[base + 16..base + 24].try_into().unwrap());
                    pts.push([x, y, z]);
                }
            }
            _ => {
                return Err(VtkError::Unsupported(format!(
                    "binary points type: {:?}",
                    scalar_type
                )))
            }
        }
        // Skip trailing newline
        let _ = self.read_line();
        Ok(pts)
    }

    fn parse_cells(&mut self, header_tokens: &[&str]) -> Result<CellArray, VtkError> {
        // <KEYWORD> <num_cells> <total_size>
        let num_cells: usize = parse_token(header_tokens.get(1), "cells count")?;
        let _total_size: usize = parse_token(header_tokens.get(2), "cells total size")?;

        if self.is_binary {
            self.parse_cells_binary(num_cells, _total_size)
        } else {
            self.parse_cells_ascii(num_cells)
        }
    }

    fn parse_cells_ascii(&mut self, num_cells: usize) -> Result<CellArray, VtkError> {
        let mut cells = CellArray::new();
        for _ in 0..num_cells {
            let line = self.read_nonempty_line()?;
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                return Err(VtkError::Parse("empty cell line".into()));
            }
            let npts: usize = tokens[0]
                .parse()
                .map_err(|_| VtkError::Parse(format!("invalid npts: {}", tokens[0])))?;
            let mut ids = Vec::with_capacity(npts);
            for t in &tokens[1..=npts] {
                let id: i64 = t
                    .parse()
                    .map_err(|_| VtkError::Parse(format!("invalid cell id: {}", t)))?;
                ids.push(id);
            }
            cells.push_cell(&ids);
        }
        Ok(cells)
    }

    fn parse_cells_binary(
        &mut self,
        num_cells: usize,
        total_size: usize,
    ) -> Result<CellArray, VtkError> {
        let mut buf = vec![0u8; total_size * 4];
        self.reader.read_exact(&mut buf)?;

        let mut cells = CellArray::new();
        let mut offset = 0;
        for _ in 0..num_cells {
            let npts = i32::from_be_bytes(buf[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            let mut ids = Vec::with_capacity(npts);
            for _ in 0..npts {
                let id = i32::from_be_bytes(buf[offset..offset + 4].try_into().unwrap()) as i64;
                offset += 4;
                ids.push(id);
            }
            cells.push_cell(&ids);
        }
        // Skip trailing newline
        let _ = self.read_line();
        Ok(cells)
    }

    fn parse_data_attributes(
        &mut self,
        attrs: &mut DataSetAttributes,
        _n: usize,
    ) -> Result<(), VtkError> {
        // Read arrays until we hit another section or EOF
        while let Ok(line) = self.peek_nonempty_line() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                break;
            }

            match tokens[0].to_uppercase().as_str() {
                "SCALARS" => {
                    // Consume the peeked line
                    let _ = self.read_nonempty_line()?;
                    let arr = self.parse_scalars(&tokens, _n)?;
                    let name = arr.name().to_string();
                    attrs.add_array(arr);
                    // Set first scalar array as active
                    if attrs.scalars().is_none() {
                        attrs.set_active_scalars(&name);
                    }
                }
                "VECTORS" | "NORMALS" => {
                    let _ = self.read_nonempty_line()?;
                    let arr = self.parse_vectors(&tokens, _n)?;
                    let name = arr.name().to_string();
                    let is_normals = tokens[0].to_uppercase() == "NORMALS";
                    attrs.add_array(arr);
                    if is_normals {
                        attrs.set_active_normals(&name);
                    } else {
                        attrs.set_active_vectors(&name);
                    }
                }
                // Stop if we hit a new section keyword
                "POINT_DATA" | "CELL_DATA" | "POINTS" | "VERTICES" | "LINES" | "POLYGONS"
                | "TRIANGLE_STRIPS" | "DATASET" => {
                    break;
                }
                _ => {
                    // Skip unknown attribute types
                    let _ = self.read_nonempty_line()?;
                }
            }
        }
        Ok(())
    }

    fn parse_scalars(
        &mut self,
        header_tokens: &[&str],
        n: usize,
    ) -> Result<AnyDataArray, VtkError> {
        // SCALARS <name> <type> [<numComp>]
        let name = header_tokens
            .get(1)
            .ok_or_else(|| VtkError::Parse("missing SCALARS name".into()))?;
        let type_name = header_tokens
            .get(2)
            .ok_or_else(|| VtkError::Parse("missing SCALARS type".into()))?;
        let num_comp: usize = header_tokens
            .get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let scalar_type = ScalarType::from_vtk_name(type_name)
            .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;

        // Read LOOKUP_TABLE line
        let lt_line = self.read_nonempty_line()?;
        if !lt_line.trim().to_uppercase().starts_with("LOOKUP_TABLE") {
            return Err(VtkError::Parse(format!(
                "expected LOOKUP_TABLE, got: {}",
                lt_line
            )));
        }

        self.read_typed_array(name, scalar_type, n, num_comp)
    }

    fn parse_vectors(
        &mut self,
        header_tokens: &[&str],
        n: usize,
    ) -> Result<AnyDataArray, VtkError> {
        // VECTORS/NORMALS <name> <type>
        let name = header_tokens
            .get(1)
            .ok_or_else(|| VtkError::Parse("missing name".into()))?;
        let type_name = header_tokens
            .get(2)
            .ok_or_else(|| VtkError::Parse("missing type".into()))?;
        let scalar_type = ScalarType::from_vtk_name(type_name)
            .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;

        self.read_typed_array(name, scalar_type, n, 3)
    }

    fn read_typed_array(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        n: usize,
        num_comp: usize,
    ) -> Result<AnyDataArray, VtkError> {
        if self.is_binary {
            return self.read_typed_array_binary(name, scalar_type, n, num_comp);
        }

        // ASCII: read n * num_comp values
        let total = n * num_comp;
        let values = self.read_f64_values(total)?;

        // Create the appropriately typed array
        match scalar_type {
            ScalarType::F32 => {
                let data: Vec<f32> = values.iter().map(|&v| v as f32).collect();
                Ok(AnyDataArray::F32(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::F64 => Ok(AnyDataArray::F64(DataArray::from_vec(
                name, values, num_comp,
            ))),
            ScalarType::I32 => {
                let data: Vec<i32> = values.iter().map(|&v| v as i32).collect();
                Ok(AnyDataArray::I32(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::I64 => {
                let data: Vec<i64> = values.iter().map(|&v| v as i64).collect();
                Ok(AnyDataArray::I64(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::U8 => {
                let data: Vec<u8> = values.iter().map(|&v| v as u8).collect();
                Ok(AnyDataArray::U8(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::I8 => {
                let data: Vec<i8> = values.iter().map(|&v| v as i8).collect();
                Ok(AnyDataArray::I8(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::I16 => {
                let data: Vec<i16> = values.iter().map(|&v| v as i16).collect();
                Ok(AnyDataArray::I16(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::U16 => {
                let data: Vec<u16> = values.iter().map(|&v| v as u16).collect();
                Ok(AnyDataArray::U16(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::U32 => {
                let data: Vec<u32> = values.iter().map(|&v| v as u32).collect();
                Ok(AnyDataArray::U32(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::U64 => {
                let data: Vec<u64> = values.iter().map(|&v| v as u64).collect();
                Ok(AnyDataArray::U64(DataArray::from_vec(name, data, num_comp)))
            }
        }
    }

    fn read_typed_array_binary(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        n: usize,
        num_comp: usize,
    ) -> Result<AnyDataArray, VtkError> {
        let total = n * num_comp;
        let byte_size = scalar_type.size();
        let mut buf = vec![0u8; total * byte_size];
        self.reader.read_exact(&mut buf)?;

        macro_rules! read_be {
            ($ty:ty, $variant:ident, $size:expr) => {{
                let data: Vec<$ty> = buf
                    .chunks_exact($size)
                    .map(|c| <$ty>::from_be_bytes(c.try_into().unwrap()))
                    .collect();
                Ok(AnyDataArray::$variant(DataArray::from_vec(
                    name, data, num_comp,
                )))
            }};
        }

        let result = match scalar_type {
            ScalarType::F32 => read_be!(f32, F32, 4),
            ScalarType::F64 => read_be!(f64, F64, 8),
            ScalarType::I16 => read_be!(i16, I16, 2),
            ScalarType::I32 => read_be!(i32, I32, 4),
            ScalarType::I64 => read_be!(i64, I64, 8),
            ScalarType::U16 => read_be!(u16, U16, 2),
            ScalarType::U32 => read_be!(u32, U32, 4),
            ScalarType::U64 => read_be!(u64, U64, 8),
            ScalarType::I8 => {
                let data: Vec<i8> = buf.iter().map(|&b| b as i8).collect();
                Ok(AnyDataArray::I8(DataArray::from_vec(name, data, num_comp)))
            }
            ScalarType::U8 => Ok(AnyDataArray::U8(DataArray::from_vec(name, buf, num_comp))),
        };

        // Skip trailing newline
        let _ = self.read_line();
        result
    }

    fn read_f64_values(&mut self, count: usize) -> Result<Vec<f64>, VtkError> {
        let mut values = Vec::with_capacity(count);
        while values.len() < count {
            let line = self.read_nonempty_line()?;
            for token in line.split_whitespace() {
                let v: f64 = token
                    .parse()
                    .map_err(|_| VtkError::Parse(format!("invalid number: {}", token)))?;
                values.push(v);
                if values.len() >= count {
                    break;
                }
            }
        }
        Ok(values)
    }

    fn read_line(&mut self) -> Result<String, VtkError> {
        let mut line = String::new();
        let bytes = self.reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(VtkError::Parse("unexpected end of file".into()));
        }
        Ok(line
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string())
    }

    fn read_nonempty_line(&mut self) -> Result<String, VtkError> {
        loop {
            let line = self.read_line()?;
            if !line.trim().is_empty() {
                return Ok(line);
            }
        }
    }

    /// Peek at the next non-empty line without consuming it.
    fn peek_nonempty_line(&mut self) -> Result<String, VtkError> {
        loop {
            let buf = self.reader.fill_buf().map_err(VtkError::Io)?;
            if buf.is_empty() {
                return Err(VtkError::Parse("unexpected end of file".into()));
            }

            // Find the next newline in the buffer
            if let Some(newline_pos) = buf.iter().position(|&b| b == b'\n') {
                let line = std::str::from_utf8(&buf[..newline_pos])
                    .map_err(|e| VtkError::Parse(format!("invalid UTF-8: {}", e)))?
                    .trim()
                    .to_string();
                if line.is_empty() {
                    // Consume the empty line and continue
                    let consume = newline_pos + 1;
                    self.reader.consume(consume);
                    continue;
                }
                return Ok(line);
            } else {
                // No newline found yet, return what we have
                let line = std::str::from_utf8(buf)
                    .map_err(|e| VtkError::Parse(format!("invalid UTF-8: {}", e)))?
                    .trim()
                    .to_string();
                if !line.is_empty() {
                    return Ok(line);
                }
                return Err(VtkError::Parse("unexpected end of file".into()));
            }
        }
    }
}

fn parse_token<T: std::str::FromStr>(token: Option<&&str>, context: &str) -> Result<T, VtkError> {
    token
        .ok_or_else(|| VtkError::Parse(format!("missing {}", context)))?
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid {}", context)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, PolyData};
    use crate::io::legacy::LegacyWriter;

    #[test]
    fn roundtrip_ascii_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let writer = LegacyWriter::ascii();
        let mut buf = Vec::new();
        writer.write_poly_data_to(&mut buf, &pd).unwrap();

        let reader_buf = std::io::BufReader::new(&buf[..]);
        let pd2 = LegacyReader::read_poly_data_from(reader_buf).unwrap();

        assert_eq!(pd2.points.len(), 3);
        assert_eq!(pd2.polys.num_cells(), 1);
        assert_eq!(pd2.polys.cell(0), &[0, 1, 2]);
        assert_eq!(pd2.points.get(0), [0.0, 0.0, 0.0]);
        assert_eq!(pd2.points.get(1), [1.0, 0.0, 0.0]);
        assert_eq!(pd2.points.get(2), [0.5, 1.0, 0.0]);
    }

    #[test]
    fn roundtrip_with_scalars() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let scalars = DataArray::from_vec("temperature", vec![100.0f64, 200.0, 300.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("temperature");

        let writer = LegacyWriter::ascii();
        let mut buf = Vec::new();
        writer.write_poly_data_to(&mut buf, &pd).unwrap();

        let reader_buf = std::io::BufReader::new(&buf[..]);
        let pd2 = LegacyReader::read_poly_data_from(reader_buf).unwrap();

        let scalars = pd2.point_data().scalars().unwrap();
        assert_eq!(scalars.name(), "temperature");
        assert_eq!(scalars.num_tuples(), 3);
        let mut val = [0.0f64];
        scalars.tuple_as_f64(0, &mut val);
        assert_eq!(val[0], 100.0);
    }

    #[test]
    fn roundtrip_binary() {
        let pd = PolyData::from_triangles(
            vec![[1.5, 2.5, 3.5], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );

        let writer = LegacyWriter::binary();
        let mut buf = Vec::new();
        writer.write_poly_data_to(&mut buf, &pd).unwrap();

        let reader_buf = std::io::BufReader::new(&buf[..]);
        let pd2 = LegacyReader::read_poly_data_from(reader_buf).unwrap();

        assert_eq!(pd2.points.len(), 3);
        assert_eq!(pd2.polys.num_cells(), 1);

        let p0 = pd2.points.get(0);
        assert!((p0[0] - 1.5).abs() < 1e-10);
        assert!((p0[1] - 2.5).abs() < 1e-10);
        assert!((p0[2] - 3.5).abs() < 1e-10);
    }
}
