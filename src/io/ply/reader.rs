use std::io::BufRead;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for Stanford PLY format (ASCII only).
pub struct PlyReader;

#[derive(Clone, Debug)]
struct PlyProperty {
    name: String,
    is_list: bool,
}

#[derive(Clone, Debug)]
struct PlyElement {
    name: String,
    count: usize,
    props: Vec<PlyProperty>,
}

impl PlyReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<PolyData, VtkError> {
        let mut lines = reader.lines();
        // Parse header
        let first = read_line(&mut lines)?;
        if first.trim() != "ply" {
            return Err(VtkError::Parse("not a PLY file".into()));
        }

        let mut in_header = true;
        let mut elements: Vec<PlyElement> = Vec::new();
        let mut current_element = None;

        while in_header {
            let line = read_line(&mut lines)?;
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "format" => {
                    // We only support ASCII
                    if parts.get(1) != Some(&"ascii") {
                        return Err(VtkError::Unsupported(
                            "only ASCII PLY format supported".into(),
                        ));
                    }
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
                                is_list: true,
                            }
                        } else {
                            PlyProperty {
                                name: parts[2].to_string(),
                                is_list: false,
                            }
                        };
                        elements[current_element.unwrap()].props.push(prop);
                    }
                }
                "end_header" => {
                    in_header = false;
                }
                _ => {} // comment, etc.
            }
        }

        let vertex_props = elements
            .iter()
            .find(|e| e.name == "vertex")
            .map(|e| e.props.as_slice())
            .ok_or_else(|| VtkError::Parse("Cannot read geometry".into()))?;
        let has_x = vertex_props.iter().any(|p| !p.is_list && p.name == "x");
        let has_y = vertex_props.iter().any(|p| !p.is_list && p.name == "y");
        let has_z = vertex_props.iter().any(|p| !p.is_list && p.name == "z");
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
        let has_alpha = vertex_props.iter().any(|p| !p.is_list && p.name == "alpha");
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
            .any(|p| !p.is_list && p.name == "intensity");
        let has_face_rgb = has_scalar_props(face_props, &["red", "green", "blue"]);
        let has_face_alpha = face_props.iter().any(|p| !p.is_list && p.name == "alpha");
        let has_face_tcoords =
            tcoord_names.is_none() && face_props.iter().any(|p| p.is_list && p.name == "texcoord");

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
                    let line = read_line(&mut lines)?;
                    let values: Vec<&str> = line.split_whitespace().collect();
                    let mut offset = 0usize;
                    let mut x = None;
                    let mut y = None;
                    let mut z = None;
                    let mut red = None;
                    let mut green = None;
                    let mut blue = None;
                    let mut alpha = None;
                    let mut nx = None;
                    let mut ny = None;
                    let mut nz = None;
                    let mut u = None;
                    let mut v = None;

                    for prop in &element.props {
                        if prop.is_list {
                            let n = parse_ascii_usize(values.get(offset).copied(), &prop.name)?;
                            offset += 1;
                            if offset + n > values.len() {
                                return Err(VtkError::Parse("truncated vertex list".into()));
                            }
                            offset += n;
                        } else {
                            let value = parse_ascii_value(values.get(offset).copied(), &prop.name)?;
                            match prop.name.as_str() {
                                "x" => x = Some(value),
                                "y" => y = Some(value),
                                "z" => z = Some(value),
                                "red" | "diffuse_red" => red = Some(value),
                                "green" | "diffuse_green" => green = Some(value),
                                "blue" | "diffuse_blue" => blue = Some(value),
                                "alpha" => alpha = Some(value),
                                "nx" => nx = Some(value),
                                "ny" => ny = Some(value),
                                "nz" => nz = Some(value),
                                "u" | "texture_u" | "s" => u = Some(value),
                                "v" | "texture_v" | "t" => v = Some(value),
                                _ => {}
                            }
                            offset += 1;
                        }
                    }

                    points.push([
                        x.ok_or_else(|| VtkError::Parse("missing vertex property x".into()))?,
                        y.ok_or_else(|| VtkError::Parse("missing vertex property y".into()))?,
                        z.ok_or_else(|| VtkError::Parse("missing vertex property z".into()))?,
                    ]);
                    if let Some(colors) = colors.as_mut() {
                        colors.push(ply_color_component(red.unwrap_or(0.0)));
                        colors.push(ply_color_component(green.unwrap_or(0.0)));
                        colors.push(ply_color_component(blue.unwrap_or(0.0)));
                        if has_alpha {
                            colors.push(ply_color_component(alpha.unwrap_or(255.0)));
                        }
                    }
                    if let Some(normals) = normals.as_mut() {
                        normals.push(nx.unwrap_or(0.0));
                        normals.push(ny.unwrap_or(0.0));
                        normals.push(nz.unwrap_or(0.0));
                    }
                    if let Some(tcoords) = tcoords.as_mut() {
                        tcoords.push(u.unwrap_or(0.0));
                        tcoords.push(v.unwrap_or(0.0));
                    }
                }
            } else if element.name == "face" {
                for _ in 0..element.count {
                    let line = read_line(&mut lines)?;
                    let values: Vec<&str> = line.split_whitespace().collect();
                    let mut offset = 0usize;
                    let mut vertex_indices = None;
                    let mut intensity = None;
                    let mut red = None;
                    let mut green = None;
                    let mut blue = None;
                    let mut alpha = None;
                    let mut texcoord = None;

                    for prop in &element.props {
                        if prop.is_list {
                            let n = parse_ascii_usize(values.get(offset).copied(), &prop.name)?;
                            offset += 1;
                            if offset + n > values.len() {
                                return Err(VtkError::Parse("truncated face list".into()));
                            }
                            if prop.name == "vertex_indices" || prop.name == "vertex_index" {
                                let mut indices = Vec::with_capacity(n);
                                for value in &values[offset..offset + n] {
                                    indices.push(value.parse::<i64>().map_err(|_| {
                                        VtkError::Parse("invalid face vertex index".into())
                                    })?);
                                }
                                vertex_indices = Some(indices);
                            } else if prop.name == "texcoord" {
                                let mut coords = Vec::with_capacity(n / 2);
                                for pair in values[offset..offset + n].chunks_exact(2) {
                                    coords.push([
                                        pair[0].parse::<f64>().map_err(|_| {
                                            VtkError::Parse(
                                                "invalid face texture coordinate".into(),
                                            )
                                        })?,
                                        pair[1].parse::<f64>().map_err(|_| {
                                            VtkError::Parse(
                                                "invalid face texture coordinate".into(),
                                            )
                                        })?,
                                    ]);
                                }
                                texcoord = Some(coords);
                            }
                            offset += n;
                        } else {
                            let value = parse_ascii_value(values.get(offset).copied(), &prop.name)?;
                            match prop.name.as_str() {
                                "intensity" => intensity = Some(value),
                                "red" => red = Some(value),
                                "green" => green = Some(value),
                                "blue" => blue = Some(value),
                                "alpha" => alpha = Some(value),
                                _ => {}
                            }
                            offset += 1;
                        }
                    }

                    if let Some(mut indices) = vertex_indices {
                        if let (Some(face_tcoords), Some(texcoord)) =
                            (face_tcoords.as_mut(), texcoord.as_deref())
                        {
                            apply_face_tcoords(
                                &mut points,
                                &mut colors,
                                if has_alpha { 4 } else { 3 },
                                &mut normals,
                                face_tcoords,
                                &mut indices,
                                texcoord,
                            )?;
                        }
                        polys.push_cell(&indices);
                        if let Some(face_intensity) = face_intensity.as_mut() {
                            face_intensity.push(ply_color_component(intensity.unwrap_or(0.0)));
                        }
                        if let Some(face_colors) = face_colors.as_mut() {
                            face_colors.push(ply_color_component(red.unwrap_or(0.0)));
                            face_colors.push(ply_color_component(green.unwrap_or(0.0)));
                            face_colors.push(ply_color_component(blue.unwrap_or(0.0)));
                            if has_face_alpha {
                                face_colors.push(ply_color_component(alpha.unwrap_or(255.0)));
                            }
                        }
                    }
                }
            } else {
                for _ in 0..element.count {
                    read_line(&mut lines)?;
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
    names
        .iter()
        .all(|name| props.iter().any(|p| !p.is_list && p.name == *name))
}

fn ply_color_component(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_ascii_value(value: Option<&str>, name: &str) -> Result<f64, VtkError> {
    value
        .ok_or_else(|| VtkError::Parse(format!("missing vertex property {name}")))?
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid vertex property {name}")))
}

fn parse_ascii_usize(value: Option<&str>, name: &str) -> Result<usize, VtkError> {
    value
        .ok_or_else(|| VtkError::Parse(format!("missing list count for {name}")))?
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid list count for {name}")))
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

fn read_line(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
) -> Result<String, VtkError> {
    lines
        .next()
        .ok_or_else(|| VtkError::Parse("unexpected end of file".into()))?
        .map_err(VtkError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ply::PlyWriter;

    #[test]
    fn roundtrip() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        PlyWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = PlyReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);

        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-6);
        assert!((p0[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn reads_face_intensity_and_colors() {
        let data = b"ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nproperty uchar intensity\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nend_header\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2 7 10 20 30 40\n";
        let reader = std::io::BufReader::new(&data[..]);
        let result = PlyReader::read_from(reader).unwrap();

        assert_eq!(result.polys.num_cells(), 1);
        assert!(result.cell_data().get_array("intensity").is_some());
        let colors = result.cell_data().get_array("RGBA").unwrap();
        assert_eq!(colors.num_components(), 4);
        assert!(result.cell_data().scalars().is_some());
    }

    #[test]
    fn reads_face_tcoords_with_point_duplication() {
        let data = b"ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 2\nproperty list uchar int vertex_indices\nproperty list uchar float texcoord\nend_header\n0 0 0\n1 0 0\n1 1 0\n0 1 0\n3 0 1 2 6 0 0 1 0 1 1\n3 0 2 3 6 0.5 0.5 1 1 0 1\n";
        let reader = std::io::BufReader::new(&data[..]);
        let result = PlyReader::read_from(reader).unwrap();

        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 5);
        assert_ne!(result.polys.cell(0)[0], result.polys.cell(1)[0]);
        assert!(result.point_data().tcoords().is_some());
    }
}
