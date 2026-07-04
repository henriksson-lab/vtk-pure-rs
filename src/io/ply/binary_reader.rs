use std::io::BufRead;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for Stanford PLY format (binary).
pub struct PlyBinaryReader;

#[derive(Clone, Copy, Debug)]
enum PlyEndian {
    Little,
    Big,
}

#[derive(Clone, Copy, Debug)]
enum PlyScalarType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

#[derive(Clone, Debug)]
struct PlyProperty {
    name: String,
    data_type: PlyScalarType,
    count_type: Option<PlyScalarType>,
}

#[derive(Clone, Debug)]
struct PlyElement {
    name: String,
    count: usize,
    props: Vec<PlyProperty>,
}

impl PlyBinaryReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(mut reader: R) -> Result<PolyData, VtkError> {
        let mut endian = None;
        let mut current_element = None;
        let mut elements: Vec<PlyElement> = Vec::new();

        // Parse ASCII header
        let mut first = String::new();
        if reader.read_line(&mut first).map_err(VtkError::Io)? == 0 {
            return Err(VtkError::Parse("unexpected end of file".into()));
        }
        if first.trim() != "ply" {
            return Err(VtkError::Parse("not a PLY file".into()));
        }

        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).map_err(VtkError::Io)? == 0 {
                return Err(VtkError::Parse("unexpected end of file".into()));
            }
            let trimmed = line.trim();

            if trimmed == "end_header" {
                break;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "format" => {
                    endian = match parts.get(1).copied() {
                        Some("binary_little_endian") => Some(PlyEndian::Little),
                        Some("binary_big_endian") => Some(PlyEndian::Big),
                        _ => None,
                    };
                }
                "element" => {
                    if parts.len() >= 3 {
                        let count: usize = parts[2]
                            .parse()
                            .map_err(|_| VtkError::Parse("invalid element count".into()))?;
                        elements.push(PlyElement {
                            name: parts[1].to_string(),
                            count,
                            props: Vec::new(),
                        });
                        current_element = Some(elements.len() - 1);
                    }
                }
                "property" => {
                    if parts.len() >= 3 && current_element.is_some() {
                        let prop = if parts[1] == "list" {
                            if parts.len() < 5 {
                                return Err(VtkError::Parse("invalid list property".into()));
                            }
                            PlyProperty {
                                name: parts[4].to_string(),
                                data_type: parse_ply_type(parts[3])?,
                                count_type: Some(parse_ply_type(parts[2])?),
                            }
                        } else {
                            PlyProperty {
                                name: parts[2].to_string(),
                                data_type: parse_ply_type(parts[1])?,
                                count_type: None,
                            }
                        };
                        elements[current_element.unwrap()].props.push(prop);
                    }
                }
                _ => {}
            }
        }

        let endian = endian.ok_or_else(|| {
            VtkError::Unsupported("only binary PLY supported by PlyBinaryReader".into())
        })?;

        let vertex_props = elements
            .iter()
            .find(|e| e.name == "vertex")
            .map(|e| e.props.as_slice())
            .ok_or_else(|| VtkError::Parse("Cannot read geometry".into()))?;
        let has_x = vertex_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "x");
        let has_y = vertex_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "y");
        let has_z = vertex_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "z");
        if !has_x || !has_y || !has_z {
            return Err(VtkError::Parse("Cannot read geometry".into()));
        }
        let rgb_names = if has_scalar_props(vertex_props, &["red", "green", "blue"]) {
            Some(["red", "green", "blue"])
        } else if has_scalar_props(
            vertex_props,
            &["diffuse_red", "diffuse_green", "diffuse_blue"],
        ) {
            Some(["diffuse_red", "diffuse_green", "diffuse_blue"])
        } else {
            None
        };
        let has_alpha = vertex_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "alpha");
        let has_normals = has_scalar_props(vertex_props, &["nx", "ny", "nz"]);
        let tcoord_names = if has_scalar_props(vertex_props, &["u", "v"]) {
            Some(["u", "v"])
        } else if has_scalar_props(vertex_props, &["texture_u", "texture_v"]) {
            Some(["texture_u", "texture_v"])
        } else if has_scalar_props(vertex_props, &["s", "t"]) {
            Some(["s", "t"])
        } else {
            None
        };
        let face_props = elements
            .iter()
            .find(|e| e.name == "face")
            .map(|e| e.props.as_slice())
            .unwrap_or(&[]);
        let has_face_intensity = face_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "intensity");
        let has_face_rgb = has_scalar_props(face_props, &["red", "green", "blue"]);
        let has_face_alpha = face_props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == "alpha");
        let has_face_tcoords = tcoord_names.is_none()
            && face_props
                .iter()
                .any(|p| p.count_type.is_some() && p.name == "texcoord");

        let mut points = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut colors = rgb_names.map(|_| Vec::<u8>::new());
        let mut normals = has_normals.then(Vec::<f64>::new);
        let mut tcoords = tcoord_names.map(|_| Vec::<f64>::new());
        let mut face_tcoords = has_face_tcoords.then(Vec::<[f64; 2]>::new);
        let mut face_intensity = has_face_intensity.then(Vec::<u8>::new);
        let mut face_colors = has_face_rgb.then(Vec::<u8>::new);

        for element in &elements {
            if element.name == "vertex" {
                for _ in 0..element.count {
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut z = 0.0;
                    let mut red = 0.0;
                    let mut green = 0.0;
                    let mut blue = 0.0;
                    let mut alpha = 255.0;
                    let mut nx = 0.0;
                    let mut ny = 0.0;
                    let mut nz = 0.0;
                    let mut u = 0.0;
                    let mut v = 0.0;
                    for prop in &element.props {
                        if let Some(count_type) = prop.count_type {
                            let n = read_binary_usize(&mut reader, count_type, endian)?;
                            for _ in 0..n {
                                skip_binary_value(&mut reader, prop.data_type)?;
                            }
                        } else {
                            let value = read_binary_f64(&mut reader, prop.data_type, endian)?;
                            match prop.name.as_str() {
                                "x" => x = value,
                                "y" => y = value,
                                "z" => z = value,
                                "red" | "diffuse_red" => red = value,
                                "green" | "diffuse_green" => green = value,
                                "blue" | "diffuse_blue" => blue = value,
                                "alpha" => alpha = value,
                                "nx" => nx = value,
                                "ny" => ny = value,
                                "nz" => nz = value,
                                "u" | "texture_u" | "s" => u = value,
                                "v" | "texture_v" | "t" => v = value,
                                _ => {}
                            }
                        }
                    }
                    points.push([x, y, z]);
                    if let Some(colors) = colors.as_mut() {
                        colors.push(ply_color_component(red));
                        colors.push(ply_color_component(green));
                        colors.push(ply_color_component(blue));
                        if has_alpha {
                            colors.push(ply_color_component(alpha));
                        }
                    }
                    if let Some(normals) = normals.as_mut() {
                        normals.push(nx);
                        normals.push(ny);
                        normals.push(nz);
                    }
                    if let Some(tcoords) = tcoords.as_mut() {
                        tcoords.push(u);
                        tcoords.push(v);
                    }
                }
            } else if element.name == "face" {
                for _ in 0..element.count {
                    let mut vertex_indices = None;
                    let mut intensity = 0.0;
                    let mut red = 0.0;
                    let mut green = 0.0;
                    let mut blue = 0.0;
                    let mut alpha = 255.0;
                    let mut texcoord = None;
                    for prop in &element.props {
                        if let Some(count_type) = prop.count_type {
                            let n = read_binary_usize(&mut reader, count_type, endian)?;
                            if prop.name == "vertex_indices" || prop.name == "vertex_index" {
                                let mut ids = Vec::with_capacity(n);
                                for _ in 0..n {
                                    ids.push(read_binary_i64(&mut reader, prop.data_type, endian)?);
                                }
                                vertex_indices = Some(ids);
                            } else if prop.name == "texcoord" {
                                let mut coords = Vec::with_capacity(n / 2);
                                for _ in 0..n / 2 {
                                    let u = read_binary_f64(&mut reader, prop.data_type, endian)?;
                                    let v = read_binary_f64(&mut reader, prop.data_type, endian)?;
                                    coords.push([u, v]);
                                }
                                if n % 2 != 0 {
                                    skip_binary_value(&mut reader, prop.data_type)?;
                                }
                                texcoord = Some(coords);
                            } else {
                                for _ in 0..n {
                                    skip_binary_value(&mut reader, prop.data_type)?;
                                }
                            }
                        } else {
                            let value = read_binary_f64(&mut reader, prop.data_type, endian)?;
                            match prop.name.as_str() {
                                "intensity" => intensity = value,
                                "red" => red = value,
                                "green" => green = value,
                                "blue" => blue = value,
                                "alpha" => alpha = value,
                                _ => {}
                            }
                        }
                    }
                    if let Some(mut ids) = vertex_indices {
                        if let (Some(face_tcoords), Some(texcoord)) =
                            (face_tcoords.as_mut(), texcoord.as_deref())
                        {
                            apply_face_tcoords(
                                &mut points,
                                &mut colors,
                                if has_alpha { 4 } else { 3 },
                                &mut normals,
                                face_tcoords,
                                &mut ids,
                                texcoord,
                            )?;
                        }
                        polys.push_cell(&ids);
                        if let Some(face_intensity) = face_intensity.as_mut() {
                            face_intensity.push(ply_color_component(intensity));
                        }
                        if let Some(face_colors) = face_colors.as_mut() {
                            face_colors.push(ply_color_component(red));
                            face_colors.push(ply_color_component(green));
                            face_colors.push(ply_color_component(blue));
                            if has_face_alpha {
                                face_colors.push(ply_color_component(alpha));
                            }
                        }
                    }
                }
            } else {
                for _ in 0..element.count {
                    for prop in &element.props {
                        if let Some(count_type) = prop.count_type {
                            let n = read_binary_usize(&mut reader, count_type, endian)?;
                            for _ in 0..n {
                                skip_binary_value(&mut reader, prop.data_type)?;
                            }
                        } else {
                            skip_binary_value(&mut reader, prop.data_type)?;
                        }
                    }
                }
            }
        }

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        if let Some(colors) = colors {
            let name = if has_alpha { "RGBA" } else { "RGB" };
            let nc = if has_alpha { 4 } else { 3 };
            pd.point_data_mut()
                .add_array(AnyDataArray::U8(DataArray::from_vec(name, colors, nc)));
            pd.point_data_mut().set_active_scalars(name);
        }
        if let Some(normals) = normals {
            pd.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "Normals", normals, 3,
                )));
            pd.point_data_mut().set_active_normals("Normals");
        }
        if let Some(tcoords) = tcoords {
            pd.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "TCoords", tcoords, 2,
                )));
            pd.point_data_mut().set_active_tcoords("TCoords");
        }
        if let Some(face_tcoords) = face_tcoords {
            let flat = face_tcoords
                .into_iter()
                .flat_map(|tc| [tc[0], tc[1]])
                .collect();
            pd.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", flat, 2)));
            pd.point_data_mut().set_active_tcoords("TCoords");
        }
        if let Some(face_intensity) = face_intensity {
            pd.cell_data_mut()
                .add_array(AnyDataArray::U8(DataArray::from_vec(
                    "intensity",
                    face_intensity,
                    1,
                )));
            pd.cell_data_mut().set_active_scalars("intensity");
        }
        if let Some(face_colors) = face_colors {
            let name = if has_face_alpha { "RGBA" } else { "RGB" };
            let nc = if has_face_alpha { 4 } else { 3 };
            pd.cell_data_mut()
                .add_array(AnyDataArray::U8(DataArray::from_vec(name, face_colors, nc)));
            pd.cell_data_mut().set_active_scalars(name);
        }
        Ok(pd)
    }
}

fn has_scalar_props(props: &[PlyProperty], names: &[&str]) -> bool {
    names.iter().all(|name| {
        props
            .iter()
            .any(|p| p.count_type.is_none() && p.name == *name)
    })
}

fn ply_color_component(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_ply_type(name: &str) -> Result<PlyScalarType, VtkError> {
    match name {
        "char" | "int8" => Ok(PlyScalarType::I8),
        "uchar" | "uint8" => Ok(PlyScalarType::U8),
        "short" | "int16" => Ok(PlyScalarType::I16),
        "ushort" | "uint16" => Ok(PlyScalarType::U16),
        "int" | "int32" => Ok(PlyScalarType::I32),
        "uint" | "uint32" => Ok(PlyScalarType::U32),
        "float" | "float32" => Ok(PlyScalarType::F32),
        "double" | "float64" => Ok(PlyScalarType::F64),
        _ => Err(VtkError::Unsupported(format!(
            "unsupported PLY type {name}"
        ))),
    }
}

fn read_binary_f64<R: BufRead>(
    reader: &mut R,
    ty: PlyScalarType,
    endian: PlyEndian,
) -> Result<f64, VtkError> {
    Ok(match ty {
        PlyScalarType::I8 => read_exact_array::<R, 1>(reader)?[0] as i8 as f64,
        PlyScalarType::U8 => read_exact_array::<R, 1>(reader)?[0] as f64,
        PlyScalarType::I16 => read_i16(reader, endian)? as f64,
        PlyScalarType::U16 => read_u16(reader, endian)? as f64,
        PlyScalarType::I32 => read_i32(reader, endian)? as f64,
        PlyScalarType::U32 => read_u32(reader, endian)? as f64,
        PlyScalarType::F32 => read_f32(reader, endian)? as f64,
        PlyScalarType::F64 => read_f64(reader, endian)?,
    })
}

fn read_binary_i64<R: BufRead>(
    reader: &mut R,
    ty: PlyScalarType,
    endian: PlyEndian,
) -> Result<i64, VtkError> {
    Ok(match ty {
        PlyScalarType::I8 => read_exact_array::<R, 1>(reader)?[0] as i8 as i64,
        PlyScalarType::U8 => read_exact_array::<R, 1>(reader)?[0] as i64,
        PlyScalarType::I16 => read_i16(reader, endian)? as i64,
        PlyScalarType::U16 => read_u16(reader, endian)? as i64,
        PlyScalarType::I32 => read_i32(reader, endian)? as i64,
        PlyScalarType::U32 => read_u32(reader, endian)? as i64,
        PlyScalarType::F32 | PlyScalarType::F64 => {
            return Err(VtkError::Unsupported(
                "floating-point face vertex indices are unsupported".into(),
            ));
        }
    })
}

fn read_binary_usize<R: BufRead>(
    reader: &mut R,
    ty: PlyScalarType,
    endian: PlyEndian,
) -> Result<usize, VtkError> {
    let value = read_binary_i64(reader, ty, endian)?;
    usize::try_from(value).map_err(|_| VtkError::Parse("negative list count".into()))
}

fn read_i16<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<i16, VtkError> {
    let bytes = read_exact_array::<R, 2>(reader)?;
    Ok(match endian {
        PlyEndian::Little => i16::from_le_bytes(bytes),
        PlyEndian::Big => i16::from_be_bytes(bytes),
    })
}

fn read_u16<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<u16, VtkError> {
    let bytes = read_exact_array::<R, 2>(reader)?;
    Ok(match endian {
        PlyEndian::Little => u16::from_le_bytes(bytes),
        PlyEndian::Big => u16::from_be_bytes(bytes),
    })
}

fn read_i32<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<i32, VtkError> {
    let bytes = read_exact_array::<R, 4>(reader)?;
    Ok(match endian {
        PlyEndian::Little => i32::from_le_bytes(bytes),
        PlyEndian::Big => i32::from_be_bytes(bytes),
    })
}

fn read_u32<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<u32, VtkError> {
    let bytes = read_exact_array::<R, 4>(reader)?;
    Ok(match endian {
        PlyEndian::Little => u32::from_le_bytes(bytes),
        PlyEndian::Big => u32::from_be_bytes(bytes),
    })
}

fn read_f32<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<f32, VtkError> {
    let bytes = read_exact_array::<R, 4>(reader)?;
    Ok(match endian {
        PlyEndian::Little => f32::from_le_bytes(bytes),
        PlyEndian::Big => f32::from_be_bytes(bytes),
    })
}

fn read_f64<R: BufRead>(reader: &mut R, endian: PlyEndian) -> Result<f64, VtkError> {
    let bytes = read_exact_array::<R, 8>(reader)?;
    Ok(match endian {
        PlyEndian::Little => f64::from_le_bytes(bytes),
        PlyEndian::Big => f64::from_be_bytes(bytes),
    })
}

fn apply_face_tcoords(
    points: &mut Points<f64>,
    colors: &mut Option<Vec<u8>>,
    color_components: usize,
    normals: &mut Option<Vec<f64>>,
    tcoords: &mut Vec<[f64; 2]>,
    ids: &mut [i64],
    new_tcoords: &[[f64; 2]],
) -> Result<(), VtkError> {
    if ids.len() != new_tcoords.len() {
        return Ok(());
    }
    if tcoords.is_empty() {
        tcoords.resize(points.len(), [-1.0, -1.0]);
    }

    for (id, &new_tex) in ids.iter_mut().zip(new_tcoords.iter()) {
        let old_id = usize::try_from(*id)
            .map_err(|_| VtkError::Parse("invalid face vertex index".into()))?;
        if old_id >= points.len() {
            return Err(VtkError::Parse("invalid face vertex index".into()));
        }
        let current_tex = tcoords[old_id];
        if current_tex == [-1.0, -1.0] {
            tcoords[old_id] = new_tex;
        } else if !fuzzy_tcoord_eq(current_tex, new_tex) {
            let point = points.get(old_id);
            if let Some(existing_id) = find_matching_point_tcoord(points, tcoords, point, new_tex) {
                *id = existing_id as i64;
            } else {
                let new_id = points.len();
                points.push(point);
                duplicate_tuple_u8(colors, old_id, color_components);
                duplicate_tuple_f64(normals, old_id, 3);
                tcoords.push(new_tex);
                *id = new_id as i64;
            }
        }
    }
    Ok(())
}

fn find_matching_point_tcoord(
    points: &Points<f64>,
    tcoords: &[[f64; 2]],
    point: [f64; 3],
    tcoord: [f64; 2],
) -> Option<usize> {
    tcoords.iter().enumerate().find_map(|(idx, &candidate)| {
        (fuzzy_tcoord_eq(candidate, tcoord) && fuzzy_point_eq(points.get(idx), point))
            .then_some(idx)
    })
}

fn duplicate_tuple_u8(values: &mut Option<Vec<u8>>, tuple: usize, components: usize) {
    if let Some(values) = values.as_mut() {
        let start = tuple * components;
        let tuple_values: Vec<u8> = values[start..start + components].to_vec();
        values.extend_from_slice(&tuple_values);
    }
}

fn duplicate_tuple_f64(values: &mut Option<Vec<f64>>, tuple: usize, components: usize) {
    if let Some(values) = values.as_mut() {
        let start = tuple * components;
        let tuple_values: Vec<f64> = values[start..start + components].to_vec();
        values.extend_from_slice(&tuple_values);
    }
}

fn fuzzy_tcoord_eq(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() <= 1e-6 && (a[1] - b[1]).abs() <= 1e-6
}

fn fuzzy_point_eq(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= 1e-6 && (a[1] - b[1]).abs() <= 1e-6 && (a[2] - b[2]).abs() <= 1e-6
}

fn skip_binary_value<R: BufRead>(reader: &mut R, ty: PlyScalarType) -> Result<(), VtkError> {
    match ty {
        PlyScalarType::I8 | PlyScalarType::U8 => {
            read_exact_array::<R, 1>(reader)?;
        }
        PlyScalarType::I16 | PlyScalarType::U16 => {
            read_exact_array::<R, 2>(reader)?;
        }
        PlyScalarType::I32 | PlyScalarType::U32 | PlyScalarType::F32 => {
            read_exact_array::<R, 4>(reader)?;
        }
        PlyScalarType::F64 => {
            read_exact_array::<R, 8>(reader)?;
        }
    }
    Ok(())
}

fn read_exact_array<R: BufRead, const N: usize>(reader: &mut R) -> Result<[u8; N], VtkError> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf).map_err(VtkError::Io)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ply::PlyBinaryWriter;

    #[test]
    fn roundtrip_binary_ply() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = PlyBinaryReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);

        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-4); // f32 precision
        assert!((p0[1] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn roundtrip_binary_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = PlyBinaryReader::read_from(reader).unwrap();

        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn reads_binary_big_endian_ply() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"ply\nformat binary_big_endian 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n",
        );
        for p in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for v in p {
                data.extend_from_slice(&v.to_be_bytes());
            }
        }
        data.push(3);
        for id in [0i32, 1, 2] {
            data.extend_from_slice(&id.to_be_bytes());
        }

        let reader = std::io::BufReader::new(&data[..]);
        let result = PlyBinaryReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert!((result.points.get(1)[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn reads_binary_face_intensity_and_colors() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"ply\nformat binary_little_endian 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nproperty uchar intensity\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nend_header\n",
        );
        for p in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for v in p {
                data.extend_from_slice(&v.to_le_bytes());
            }
        }
        data.push(3);
        for id in [0i32, 1, 2] {
            data.extend_from_slice(&id.to_le_bytes());
        }
        data.extend_from_slice(&[7, 10, 20, 30, 40]);

        let reader = std::io::BufReader::new(&data[..]);
        let result = PlyBinaryReader::read_from(reader).unwrap();

        assert_eq!(result.polys.num_cells(), 1);
        assert!(result.cell_data().get_array("intensity").is_some());
        let colors = result.cell_data().get_array("RGBA").unwrap();
        assert_eq!(colors.num_components(), 4);
        assert!(result.cell_data().scalars().is_some());
    }

    #[test]
    fn reads_binary_face_tcoords_with_point_duplication() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"ply\nformat binary_little_endian 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 2\nproperty list uchar int vertex_indices\nproperty list uchar float texcoord\nend_header\n",
        );
        for p in [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ] {
            for v in p {
                data.extend_from_slice(&v.to_le_bytes());
            }
        }
        for (ids, tcoords) in [
            ([0i32, 1, 2], [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0]]),
            ([0i32, 2, 3], [[0.5f32, 0.5], [1.0, 1.0], [0.0, 1.0]]),
        ] {
            data.push(3);
            for id in ids {
                data.extend_from_slice(&id.to_le_bytes());
            }
            data.push(6);
            for tcoord in tcoords {
                for v in tcoord {
                    data.extend_from_slice(&v.to_le_bytes());
                }
            }
        }

        let reader = std::io::BufReader::new(&data[..]);
        let result = PlyBinaryReader::read_from(reader).unwrap();

        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 5);
        assert_ne!(result.polys.cell(0)[0], result.polys.cell(1)[0]);
        assert!(result.point_data().tcoords().is_some());
    }
}
