use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData, StringArray};
use crate::types::VtkError;

/// Reader for Wavefront OBJ format.
pub struct ObjReader;

impl ObjReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<PolyData, VtkError> {
        let mut point_values = Vec::<[f64; 3]>::new();
        let mut normal_values = Vec::<[f64; 3]>::new();
        let mut tcoord_values = Vec::<[f64; 2]>::new();

        let mut verts = CellArray::new();
        let mut lines = CellArray::new();
        let mut polys = CellArray::new();
        let mut faces = Vec::<ObjFace>::new();
        let mut face_group_ids = Vec::new();
        let mut material_names = Vec::<String>::new();
        let mut material_libraries = Vec::<String>::new();
        let mut material_name_to_id = HashMap::<String, i32>::new();
        let mut start_cell_to_material_id = HashMap::<usize, i32>::new();
        let mut tcoords_map = Vec::<MaterialTCoords>::new();

        let mut normals_match_vertices = true;
        let mut tcoords_match_vertices = true;
        let mut has_face_tcoords = false;
        let mut group_id = -1.0f64;
        let mut tcoords_name = String::new();
        let mut cell_with_not_texture_found = false;
        let mut pending = String::new();

        for line in reader.lines() {
            let line = line.map_err(VtkError::Io)?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(prefix) = trimmed.strip_suffix('\\') {
                pending.push_str(prefix);
                pending.push(' ');
                continue;
            }

            let logical_line;
            let trimmed = if pending.is_empty() {
                trimmed
            } else {
                pending.push_str(trimmed);
                logical_line = pending.clone();
                pending.clear();
                logical_line.trim()
            };

            let mut parts = trimmed.split_whitespace();
            let Some(keyword) = parts.next() else {
                continue;
            };

            match keyword {
                "v" => {
                    let coords = parse_f64s(parts, 3, "vertex")?;
                    point_values.push([coords[0], coords[1], coords[2]]);
                }
                "vn" => {
                    let coords = parse_f64s(parts, 3, "normal")?;
                    normal_values.push([coords[0], coords[1], coords[2]]);
                }
                "g" => {
                    group_id += 1.0;
                }
                "usemtl" => {
                    let name = parts
                        .next()
                        .ok_or_else(|| VtkError::Parse("failed to parse material name".into()))?
                        .to_string();
                    let material_id =
                        *material_name_to_id.entry(name.clone()).or_insert_with(|| {
                            let id = material_names.len() as i32;
                            material_names.push(name.clone());
                            id
                        });
                    if find_tcoords_map(&tcoords_map, &name).is_none() {
                        tcoords_map.push(MaterialTCoords {
                            name: name.clone(),
                            used: Vec::new(),
                        });
                    }
                    tcoords_name = name;
                    start_cell_to_material_id.insert(polys.num_cells(), material_id);
                }
                "mtllib" => {
                    let name = parts.next().ok_or_else(|| {
                        VtkError::Parse("failed to parse material lib name".into())
                    })?;
                    material_libraries.push(name.to_string());
                }
                "vt" => {
                    let coords = parse_f64s(parts, 2, "texture coordinate")?;
                    tcoord_values.push([coords[0], coords[1]]);
                }
                "p" => {
                    let ids = parse_obj_index_list(parts, point_values.len(), "point")?;
                    if ids.is_empty() {
                        return Err(VtkError::Parse("empty `p` command in OBJ file".into()));
                    }
                    verts.push_cell(&ids);
                }
                "l" => {
                    let ids = parse_obj_line(parts, point_values.len())?;
                    if ids.len() < 2 {
                        return Err(VtkError::Parse("empty `l` command in OBJ file".into()));
                    }
                    lines.push_cell(&ids);
                }
                "f" => {
                    let face = parse_obj_face(
                        parts,
                        point_values.len(),
                        tcoord_values.len(),
                        normal_values.len(),
                    )?;
                    if face.vertices.len() < 3 {
                        return Err(VtkError::Parse(
                            "definition of a face needs at least 3 vertices".into(),
                        ));
                    }
                    if !cell_with_not_texture_found {
                        cell_with_not_texture_found = true;
                        let material_id = ensure_material(
                            "NO_MATERIAL",
                            &mut material_names,
                            &mut material_name_to_id,
                        );
                        start_cell_to_material_id.insert(polys.num_cells(), material_id);
                    }
                    has_face_tcoords |= face.tcoords.iter().any(Option::is_some);
                    for ((&vertex, &tcoord), &normal) in
                        face.vertices.iter().zip(&face.tcoords).zip(&face.normals)
                    {
                        if let Some(tcoord) = tcoord {
                            if tcoords_map.is_empty() {
                                tcoords_name = "TCoords".to_string();
                                tcoords_map.push(MaterialTCoords {
                                    name: tcoords_name.clone(),
                                    used: Vec::new(),
                                });
                            }
                            let map_index = find_tcoords_map(&tcoords_map, &tcoords_name)
                                .expect("active tcoords map must exist");
                            if tcoord >= tcoords_map[map_index].used.len() {
                                tcoords_map[map_index].used.resize(tcoord + 1, false);
                            }
                            tcoords_map[map_index].used[tcoord] = true;
                            tcoords_match_vertices &= tcoord == vertex as usize;
                        }
                        if let Some(normal) = normal {
                            normals_match_vertices &= normal == vertex as usize;
                        }
                    }
                    polys.push_cell(&face.vertices);
                    group_id = group_id.max(0.0);
                    face_group_ids.push(group_id);
                    faces.push(face);
                }
                _ => {
                    // Skip unsupported commands: s, o, etc.
                }
            }
        }

        let mut pd = PolyData::new();
        let need_fix = !normals_match_vertices || !tcoords_match_vertices;

        if need_fix {
            let mut fixed_points = Vec::with_capacity(polys.connectivity_len());
            let mut fixed_polys = CellArray::new();
            let mut next_vertex = 0i64;

            for face in &faces {
                let mut cell = Vec::with_capacity(face.vertices.len());
                for i in 0..face.vertices.len() {
                    fixed_points.push(point_values[face.vertices[i] as usize]);
                    cell.push(next_vertex);
                    next_vertex += 1;
                }
                fixed_polys.push_cell(&cell);
            }

            pd.points = Points::from_vec(fixed_points);
            pd.polys = fixed_polys;
        } else {
            pd.points = Points::from_vec(point_values);
            pd.verts = verts;
            pd.lines = lines;
            pd.polys = polys;
        }

        if !normal_values.is_empty() {
            let data = if need_fix {
                build_fixed_normals(&faces, &normal_values)
            } else {
                normal_values
                    .iter()
                    .flat_map(|n| n.iter().copied())
                    .collect()
            };
            let normals = DataArray::from_vec("Normals", data, 3);
            pd.point_data_mut().add_array(AnyDataArray::F64(normals));
            pd.point_data_mut().set_active_normals("Normals");
        }

        if has_face_tcoords {
            let active_tcoords = if tcoords_map.is_empty() {
                vec![MaterialTCoords {
                    name: "TCoords".to_string(),
                    used: Vec::new(),
                }]
            } else {
                tcoords_map
            };

            for tcoords_entry in &active_tcoords {
                let data = if need_fix {
                    build_fixed_tcoords(&faces, &tcoord_values, Some(&tcoords_entry.used))
                } else {
                    build_point_tcoords(
                        pd.points.len(),
                        &faces,
                        &tcoord_values,
                        Some(&tcoords_entry.used),
                    )
                };
                let tcoords = DataArray::from_vec(&tcoords_entry.name, data, 2);
                pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
            }
            pd.point_data_mut()
                .set_active_tcoords(&active_tcoords[0].name);
        }

        if !face_group_ids.is_empty() {
            pd.cell_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "GroupIds",
                    face_group_ids,
                    1,
                )));
        }

        let has_material =
            material_names.len() > 1 || material_names.first().is_some_and(|m| m != "NO_MATERIAL");
        if has_material {
            let material_ids = build_material_ids(faces.len(), &start_cell_to_material_id);
            if material_ids.len() == faces.len() {
                pd.cell_data_mut()
                    .add_array(AnyDataArray::I32(DataArray::from_vec(
                        "MaterialIds",
                        material_ids,
                        1,
                    )));
            }
            pd.field_data_mut()
                .add_string_array(StringArray::from_vec("MaterialNames", material_names));
            if !material_libraries.is_empty() {
                pd.field_data_mut().add_string_array(StringArray::from_vec(
                    "MaterialLibraries",
                    material_libraries,
                ));
            }
        }

        Ok(pd)
    }
}

#[derive(Debug)]
struct MaterialTCoords {
    name: String,
    used: Vec<bool>,
}

#[derive(Debug)]
struct ObjFace {
    vertices: Vec<i64>,
    tcoords: Vec<Option<usize>>,
    normals: Vec<Option<usize>>,
}

fn ensure_material(
    name: &str,
    material_names: &mut Vec<String>,
    material_name_to_id: &mut HashMap<String, i32>,
) -> i32 {
    if let Some(id) = material_name_to_id.get(name) {
        *id
    } else {
        let id = material_names.len() as i32;
        material_names.push(name.to_string());
        material_name_to_id.insert(name.to_string(), id);
        id
    }
}

fn find_tcoords_map(tcoords_map: &[MaterialTCoords], name: &str) -> Option<usize> {
    tcoords_map.iter().position(|entry| entry.name == name)
}

fn build_material_ids(
    face_count: usize,
    start_cell_to_material_id: &HashMap<usize, i32>,
) -> Vec<i32> {
    let mut starts: Vec<(usize, i32)> = start_cell_to_material_id
        .iter()
        .map(|(&cell, &material_id)| (cell, material_id))
        .collect();
    starts.sort_by_key(|&(cell, _)| cell);

    let mut material_ids = Vec::with_capacity(face_count);
    let mut start_index = 0usize;
    let mut material_id = 0i32;
    for cell in 0..face_count {
        while start_index < starts.len() && starts[start_index].0 == cell {
            material_id = starts[start_index].1;
            start_index += 1;
        }
        material_ids.push(material_id);
    }
    material_ids
}

fn parse_f64s<'a>(
    parts: impl Iterator<Item = &'a str>,
    min_count: usize,
    what: &str,
) -> Result<Vec<f64>, VtkError> {
    let values: Result<Vec<_>, _> = parts.map(str::parse::<f64>).collect();
    let values = values.map_err(|_| VtkError::Parse(format!("failed to parse {what} value")))?;
    if values.len() < min_count {
        return Err(VtkError::Parse(format!(
            "expected at least {min_count} {what} values"
        )));
    }
    Ok(values)
}

fn obj_index(index: i64, count: usize, what: &str) -> Result<usize, VtkError> {
    let absolute = if index < 0 {
        count as i64 + index
    } else {
        index - 1
    };
    if absolute < 0 || absolute as usize >= count {
        return Err(VtkError::Parse(format!(
            "unexpected {what} index value: {index}"
        )));
    }
    Ok(absolute as usize)
}

fn parse_obj_index_list<'a>(
    parts: impl Iterator<Item = &'a str>,
    point_count: usize,
    what: &str,
) -> Result<Vec<i64>, VtkError> {
    parts
        .map(|part| {
            let index = part
                .parse::<i64>()
                .map_err(|_| VtkError::Parse(format!("unexpected token in OBJ {what}")))?;
            Ok(obj_index(index, point_count, what)? as i64)
        })
        .collect()
}

fn parse_obj_line<'a>(
    parts: impl Iterator<Item = &'a str>,
    point_count: usize,
) -> Result<Vec<i64>, VtkError> {
    parts
        .map(|part| {
            let vertex = part.split('/').next().unwrap_or_default();
            let index = vertex
                .parse::<i64>()
                .map_err(|_| VtkError::Parse("unexpected token in OBJ line".into()))?;
            Ok(obj_index(index, point_count, "line point")? as i64)
        })
        .collect()
}

fn parse_obj_face<'a>(
    parts: impl Iterator<Item = &'a str>,
    point_count: usize,
    tcoord_count: usize,
    normal_count: usize,
) -> Result<ObjFace, VtkError> {
    let mut vertices = Vec::new();
    let mut tcoords = Vec::new();
    let mut normals = Vec::new();

    for part in parts {
        let fields: Vec<&str> = part.split('/').collect();
        let vertex = fields
            .first()
            .copied()
            .unwrap_or_default()
            .parse::<i64>()
            .map_err(|_| VtkError::Parse("unexpected token in OBJ face".into()))?;
        vertices.push(obj_index(vertex, point_count, "point")? as i64);

        let tcoord = match fields.get(1).copied() {
            Some("") | None => None,
            Some(value) => Some(obj_index(
                value
                    .parse::<i64>()
                    .map_err(|_| VtkError::Parse("invalid token after / in OBJ file".into()))?,
                tcoord_count,
                "texture coordinate",
            )?),
        };
        let normal = match fields.get(2).copied() {
            Some("") | None => None,
            Some(value) => Some(obj_index(
                value
                    .parse::<i64>()
                    .map_err(|_| VtkError::Parse("invalid token after // in OBJ file".into()))?,
                normal_count,
                "normal",
            )?),
        };

        tcoords.push(tcoord);
        normals.push(normal);
    }

    let tcoord_count = tcoords.iter().filter(|v| v.is_some()).count();
    let normal_count = normals.iter().filter(|v| v.is_some()).count();
    if (tcoord_count > 0 && tcoord_count != vertices.len())
        || (normal_count > 0 && normal_count != vertices.len())
    {
        return Err(VtkError::Parse(
            "definition of a face must match for all points".into(),
        ));
    }

    Ok(ObjFace {
        vertices,
        tcoords,
        normals,
    })
}

fn build_fixed_normals(faces: &[ObjFace], normals: &[[f64; 3]]) -> Vec<f64> {
    let mut data = Vec::new();
    for face in faces {
        for normal in &face.normals {
            let n = normal.map(|idx| normals[idx]).unwrap_or([0.0, 0.0, 0.0]);
            data.extend_from_slice(&n);
        }
    }
    data
}

fn build_fixed_tcoords(
    faces: &[ObjFace],
    tcoords: &[[f64; 2]],
    used_tcoords: Option<&[bool]>,
) -> Vec<f64> {
    let mut data = Vec::new();
    for face in faces {
        for tcoord in &face.tcoords {
            let t = tcoord
                .filter(|&idx| used_tcoords.is_none_or(|used| idx < used.len() && used[idx]))
                .map(|idx| tcoords[idx])
                .unwrap_or([-1.0, -1.0]);
            data.extend_from_slice(&t);
        }
    }
    data
}

fn build_point_tcoords(
    point_count: usize,
    faces: &[ObjFace],
    tcoords: &[[f64; 2]],
    used_tcoords: Option<&[bool]>,
) -> Vec<f64> {
    let mut data = vec![-1.0; point_count * 2];
    for face in faces {
        for (&vertex, &tcoord) in face.vertices.iter().zip(&face.tcoords) {
            if let Some(tcoord) =
                tcoord.filter(|&idx| used_tcoords.is_none_or(|used| idx < used.len() && used[idx]))
            {
                let offset = vertex as usize * 2;
                data[offset] = tcoords[tcoord][0];
                data[offset + 1] = tcoords[tcoord][1];
            }
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::obj::ObjWriter;

    #[test]
    fn roundtrip() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        ObjWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = ObjReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn parse_complex_faces() {
        let obj = b"# test\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nvn 0 0 1\nvn 0 0 1\nvn 0 0 1\nvn 0 0 1\nf 1/1/1 2/2/2 3/3/3 4/4/4\n";
        let reader = std::io::BufReader::new(&obj[..]);
        let result = ObjReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn parses_material_names_libraries_and_ids() {
        let obj = b"mtllib materials.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nusemtl mat_a\nf 1/1 2/2 3/3\n";
        let reader = std::io::BufReader::new(&obj[..]);
        let result = ObjReader::read_from(reader).unwrap();

        let material_names = result
            .field_data()
            .get_string_array("MaterialNames")
            .unwrap();
        assert_eq!(
            material_names.as_slice(),
            &["mat_a".to_string(), "NO_MATERIAL".to_string()]
        );
        let material_libraries = result
            .field_data()
            .get_string_array("MaterialLibraries")
            .unwrap();
        assert_eq!(
            material_libraries.as_slice(),
            &["materials.mtl".to_string()]
        );
        assert!(result.point_data().get_array("mat_a").is_some());

        let material_ids = result.cell_data().get_array("MaterialIds").unwrap();
        assert_eq!(material_ids.to_f64_vec(), vec![1.0]);
    }

    #[test]
    fn preserves_normals_when_tcoord_remap_duplicates_points() {
        let obj =
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 0.5 0\nvt 0 0.5\nvn 0 0 1\nf 1/2 2/3 3/1\n";
        let reader = std::io::BufReader::new(&obj[..]);
        let result = ObjReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        let normals = result.point_data().normals().unwrap();
        assert_eq!(normals.num_tuples(), 3);
        assert_eq!(normals.to_f64_vec_flat(), vec![0.0; 9]);
    }
}
