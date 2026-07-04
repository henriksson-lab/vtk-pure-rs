//! Tecplot ASCII data format reader and writer for vtk-rs.
//!
//! Supports Tecplot ASCII `.dat` files with POINT and BLOCK data packing,
//! and FE (finite element) zone types (TRIANGLE, QUADRILATERAL, TETRAHEDRON).

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::io::{BufRead, Write};

/// Write a PolyData mesh as a Tecplot ASCII file.
pub fn write_tecplot<W: Write>(w: &mut W, mesh: &PolyData, title: &str) -> std::io::Result<()> {
    let n_pts = mesh.points.len();
    let mut tris: Vec<[i64; 3]> = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 1..cell.len() - 1 {
            tris.push([cell[0], cell[i], cell[i + 1]]);
        }
    }
    let n_cells = tris.len();

    writeln!(w, "TITLE = \"{title}\"")?;
    writeln!(w, "VARIABLES = \"X\" \"Y\" \"Z\"")?;
    writeln!(
        w,
        "ZONE T=\"Zone1\", N={n_pts}, E={n_cells}, F=FEPOINT, ET=TRIANGLE"
    )?;

    // Point data
    for i in 0..n_pts {
        let p = mesh.points.get(i);
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
    }

    // Connectivity (1-based)
    for tri in tris {
        writeln!(w, "{} {} {}", tri[0] + 1, tri[1] + 1, tri[2] + 1)?;
    }

    Ok(())
}

/// Read a Tecplot ASCII file into PolyData.
pub fn read_tecplot<R: BufRead>(mut reader: R) -> Result<PolyData, String> {
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let tokens = tokenize_tecplot(&text);
    let mut pos = 0usize;

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut var_names: Vec<String> = Vec::new();
    let mut cell_based: Vec<bool> = Vec::new();
    let mut point_extra_names: Vec<String> = Vec::new();
    let mut point_extra_data: Vec<Vec<f64>> = Vec::new();
    let mut cell_extra_names: Vec<String> = Vec::new();
    let mut cell_extra_data: Vec<Vec<f64>> = Vec::new();

    while pos < tokens.len() {
        let tok = tokens[pos].to_uppercase();
        pos += 1;

        if tok == "TITLE" {
            pos += usize::from(pos < tokens.len());
        } else if tok == "VARIABLES" {
            var_names.clear();
            while pos < tokens.len() {
                let upper = tokens[pos].to_uppercase();
                if upper == "ZONE" || upper == "TITLE" {
                    break;
                }
                var_names.push(tokens[pos].clone());
                pos += 1;
            }
            let coord = coordinate_indices(&var_names);
            cell_based = vec![false; var_names.len()];
            point_extra_names = var_names
                .iter()
                .enumerate()
                .filter(|(i, _)| !coord.contains(&Some(*i)))
                .map(|(_, name)| name.clone())
                .collect();
            point_extra_data = vec![Vec::new(); point_extra_names.len()];
            cell_extra_names.clear();
            cell_extra_data.clear();
        } else if tok == "ZONE" {
            let mut n_nodes = 0usize;
            let mut n_elements = 0usize;
            let mut format = String::new();
            let mut elem_type = String::from("TRIANGLE");
            let mut zone_type = String::new();

            while pos < tokens.len() && !is_numeric_token(&tokens[pos]) {
                let key = tokens[pos].to_uppercase();
                pos += 1;
                match key.as_str() {
                    "T" | "STRANDID" | "SOLUTIONTIME" | "PARENTZONE" | "AUXDATA" | "DT" => {
                        pos += usize::from(pos < tokens.len());
                    }
                    "N" | "NODES" => n_nodes = parse_usize(next_token(&tokens, &mut pos)?)?,
                    "E" | "ELEMENTS" => n_elements = parse_usize(next_token(&tokens, &mut pos)?)?,
                    "F" | "DATAPACKING" => format = next_token(&tokens, &mut pos)?.to_uppercase(),
                    "ET" => elem_type = next_token(&tokens, &mut pos)?.to_uppercase(),
                    "ZONETYPE" => zone_type = next_token(&tokens, &mut pos)?.to_uppercase(),
                    "VARLOCATION" => {
                        parse_varlocation(&tokens, &mut pos, &mut cell_based)?;
                    }
                    _ => {}
                }
            }

            if n_nodes == 0 {
                return Err("Tecplot zone missing N/NODES".into());
            }
            let packing = if format.is_empty() {
                if zone_type.starts_with("FE") {
                    "FEPOINT"
                } else {
                    "POINT"
                }
            } else {
                format.as_str()
            };

            let coord = coordinate_indices(&var_names);
            let first_point = points.len();
            match packing {
                "FEPOINT" | "POINT" => {
                    let point_based = vec![false; var_names.len()];
                    rebuild_extra_arrays(
                        &var_names,
                        &coord,
                        &point_based,
                        &mut point_extra_names,
                        &mut point_extra_data,
                        &mut cell_extra_names,
                        &mut cell_extra_data,
                    );
                    read_point_packing(
                        &tokens,
                        &mut pos,
                        n_nodes,
                        &coord,
                        &mut points,
                        &mut point_extra_data,
                        &var_names,
                    )?;
                }
                "FEBLOCK" | "BLOCK" => {
                    rebuild_extra_arrays(
                        &var_names,
                        &coord,
                        &cell_based,
                        &mut point_extra_names,
                        &mut point_extra_data,
                        &mut cell_extra_names,
                        &mut cell_extra_data,
                    );
                    read_block_packing(
                        &tokens,
                        &mut pos,
                        n_nodes,
                        n_elements,
                        &coord,
                        &cell_based,
                        &mut points,
                        &mut point_extra_data,
                        &mut cell_extra_data,
                        &var_names,
                    )?;
                }
                other => return Err(format!("unsupported Tecplot data packing '{other}'")),
            }

            let nodes_per_element = nodes_per_element(&elem_type, &zone_type);
            for _ in 0..n_elements {
                let mut ids = Vec::with_capacity(nodes_per_element);
                for _ in 0..nodes_per_element {
                    let id = parse_i64(next_token(&tokens, &mut pos)?)? - 1 + first_point as i64;
                    ids.push(id);
                }
                if ids.len() >= 3 {
                    polys.push_cell(&ids);
                }
            }
        }
    }

    let point_count = points.len();
    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;

    // Add extra variables as point data
    for (i, data) in point_extra_data.into_iter().enumerate() {
        if data.len() == point_count {
            let name = point_extra_names
                .get(i)
                .map(String::as_str)
                .unwrap_or("var");
            mesh.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
        }
    }
    let cell_count = mesh.polys.num_cells();
    for (i, data) in cell_extra_data.into_iter().enumerate() {
        if data.len() == cell_count {
            let name = cell_extra_names.get(i).map(String::as_str).unwrap_or("var");
            mesh.cell_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
        }
    }

    Ok(mesh)
}

fn tokenize_tecplot(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' {
            for ch in chars.by_ref() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
            }
            continue;
        }
        if is_tecplot_separator(c) {
            continue;
        }
        if c == '"' {
            let mut token = String::new();
            for ch in chars.by_ref() {
                if ch == '"' {
                    break;
                }
                token.push(ch);
            }
            if !token.is_empty() {
                tokens.push(token);
            }
        } else {
            let mut token = String::from(c);
            while let Some(&ch) = chars.peek() {
                if is_tecplot_separator(ch) {
                    break;
                }
                token.push(ch);
                chars.next();
            }
            tokens.push(token);
        }
    }
    tokens
}

fn is_tecplot_separator(c: char) -> bool {
    c.is_whitespace() || matches!(c, ',' | '=' | '(' | ')')
}

fn next_token<'a>(tokens: &'a [String], pos: &mut usize) -> Result<&'a str, String> {
    let token = tokens
        .get(*pos)
        .map(String::as_str)
        .ok_or_else(|| "unexpected end of Tecplot input".to_string())?;
    *pos += 1;
    Ok(token)
}

fn parse_usize(token: &str) -> Result<usize, String> {
    token
        .parse()
        .map_err(|_| format!("invalid Tecplot integer '{token}'"))
}

fn parse_i64(token: &str) -> Result<i64, String> {
    token
        .parse()
        .map_err(|_| format!("invalid Tecplot connectivity id '{token}'"))
}

fn parse_f64(token: &str) -> Result<f64, String> {
    token
        .parse()
        .map_err(|_| format!("invalid Tecplot value '{token}'"))
}

fn is_numeric_token(token: &str) -> bool {
    token
        .as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.'))
}

fn coordinate_indices(var_names: &[String]) -> [Option<usize>; 3] {
    let mut coord = [None, None, None];
    for (i, name) in var_names.iter().enumerate() {
        let n = name.trim().to_ascii_lowercase();
        match n.as_str() {
            "x" | "x[m]" | "x-coordinate" | "x coordinate" => coord[0] = Some(i),
            "y" | "y[m]" | "y-coordinate" | "y coordinate" => coord[1] = Some(i),
            "z" | "z[m]" | "z-coordinate" | "z coordinate" => coord[2] = Some(i),
            _ => {}
        }
    }
    for (axis, slot) in coord.iter_mut().enumerate() {
        if slot.is_none() && var_names.len() > axis {
            *slot = Some(axis);
        }
    }
    coord
}

fn read_point_packing(
    tokens: &[String],
    pos: &mut usize,
    n_nodes: usize,
    coord: &[Option<usize>; 3],
    points: &mut Points<f64>,
    extra_data: &mut [Vec<f64>],
    var_names: &[String],
) -> Result<(), String> {
    for _ in 0..n_nodes {
        let mut vals = Vec::with_capacity(var_names.len());
        for _ in 0..var_names.len() {
            vals.push(parse_f64(next_token(tokens, pos)?)?);
        }
        push_point_and_arrays(&vals, coord, points, extra_data);
    }
    Ok(())
}

fn read_block_packing(
    tokens: &[String],
    pos: &mut usize,
    n_nodes: usize,
    n_elements: usize,
    coord: &[Option<usize>; 3],
    cell_based: &[bool],
    points: &mut Points<f64>,
    point_extra_data: &mut [Vec<f64>],
    cell_extra_data: &mut [Vec<f64>],
    var_names: &[String],
) -> Result<(), String> {
    let mut blocks = Vec::with_capacity(var_names.len());
    for v in 0..var_names.len() {
        let is_coord = coord.contains(&Some(v));
        let count = if !is_coord && cell_based.get(v).copied().unwrap_or(false) {
            n_elements
        } else {
            n_nodes
        };
        let mut block = vec![0.0; count];
        for value in &mut block {
            *value = parse_f64(next_token(tokens, pos)?)?;
        }
        blocks.push(block);
    }
    for n in 0..n_nodes {
        let vals: Vec<f64> = blocks
            .iter()
            .enumerate()
            .map(|(v, block)| {
                if coord.contains(&Some(v)) || !cell_based.get(v).copied().unwrap_or(false) {
                    block[n]
                } else {
                    0.0
                }
            })
            .collect();
        push_point_and_arrays(&vals, coord, points, point_extra_data);
    }
    let mut cell_extra = 0;
    for (v, block) in blocks.iter().enumerate() {
        if coord.contains(&Some(v)) || !cell_based.get(v).copied().unwrap_or(false) {
            continue;
        }
        if let Some(data) = cell_extra_data.get_mut(cell_extra) {
            data.extend_from_slice(block);
        }
        cell_extra += 1;
    }
    Ok(())
}

fn rebuild_extra_arrays(
    var_names: &[String],
    coord: &[Option<usize>; 3],
    cell_based: &[bool],
    point_extra_names: &mut Vec<String>,
    point_extra_data: &mut Vec<Vec<f64>>,
    cell_extra_names: &mut Vec<String>,
    cell_extra_data: &mut Vec<Vec<f64>>,
) {
    point_extra_names.clear();
    point_extra_data.clear();
    cell_extra_names.clear();
    cell_extra_data.clear();
    for (i, name) in var_names.iter().enumerate() {
        if coord.contains(&Some(i)) {
            continue;
        }
        if cell_based.get(i).copied().unwrap_or(false) {
            cell_extra_names.push(name.clone());
            cell_extra_data.push(Vec::new());
        } else {
            point_extra_names.push(name.clone());
            point_extra_data.push(Vec::new());
        }
    }
}

fn parse_varlocation(
    tokens: &[String],
    pos: &mut usize,
    cell_based: &mut [bool],
) -> Result<(), String> {
    let mut explicit_vars: Vec<String> = Vec::new();
    let mut sequence_index = 0usize;
    while *pos < tokens.len() {
        let raw = tokens[*pos].trim();
        let upper = raw.trim_matches(|c| matches!(c, '[' | ']')).to_uppercase();
        if matches!(
            upper.as_str(),
            "ZONE"
                | "TITLE"
                | "VARIABLES"
                | "T"
                | "I"
                | "J"
                | "K"
                | "N"
                | "NODES"
                | "E"
                | "ELEMENTS"
                | "F"
                | "DATAPACKING"
                | "ET"
                | "ZONETYPE"
        ) {
            break;
        }
        *pos += 1;

        match upper.as_str() {
            "NODAL" => {
                sequence_index += 1;
                if sequence_index >= cell_based.len() {
                    break;
                }
            }
            "CELLCENTERED" => {
                if explicit_vars.is_empty() {
                    if let Some(slot) = cell_based.get_mut(sequence_index) {
                        *slot = true;
                    }
                    sequence_index += 1;
                    if sequence_index >= cell_based.len() {
                        break;
                    }
                } else {
                    for part in explicit_vars.drain(..) {
                        mark_cell_based_range(&part, cell_based)?;
                    }
                    break;
                }
            }
            _ => {
                if upper.chars().all(|c| c.is_ascii_digit() || c == '-') {
                    explicit_vars.push(upper);
                }
            }
        }
    }
    Ok(())
}

fn mark_cell_based_range(part: &str, cell_based: &mut [bool]) -> Result<(), String> {
    if let Some((start, end)) = part.split_once('-') {
        let start = parse_usize(start)?.saturating_sub(1);
        let end = parse_usize(end)?.saturating_sub(1);
        for idx in start..=end {
            if let Some(slot) = cell_based.get_mut(idx) {
                *slot = true;
            }
        }
    } else {
        let idx = parse_usize(part)?.saturating_sub(1);
        if let Some(slot) = cell_based.get_mut(idx) {
            *slot = true;
        }
    }
    Ok(())
}

fn push_point_and_arrays(
    vals: &[f64],
    coord: &[Option<usize>; 3],
    points: &mut Points<f64>,
    extra_data: &mut [Vec<f64>],
) {
    let mut p = [0.0; 3];
    for axis in 0..3 {
        if let Some(idx) = coord[axis].filter(|&idx| idx < vals.len()) {
            p[axis] = vals[idx];
        }
    }
    points.push(p);

    let mut extra = 0;
    for i in 0..vals.len() {
        if coord.contains(&Some(i)) {
            continue;
        }
        if let Some(data) = extra_data.get_mut(extra) {
            data.push(vals[i]);
        }
        extra += 1;
    }
}

fn nodes_per_element(elem_type: &str, zone_type: &str) -> usize {
    let kind = if !elem_type.is_empty() {
        elem_type
    } else {
        zone_type.strip_prefix("FE").unwrap_or(zone_type)
    };
    match kind {
        "TRIANGLE" | "FETRIANGLE" => 3,
        "QUADRILATERAL" | "FEQUADRILATERAL" => 4,
        "TETRAHEDRON" | "FETETRAHEDRON" => 4,
        "BRICK" | "FEBRICK" => 8,
        _ => 3,
    }
}

pub fn read_tecplot_file(path: &std::path::Path) -> Result<PolyData, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_tecplot(std::io::BufReader::new(f))
}

pub fn write_tecplot_file(
    mesh: &PolyData,
    path: &std::path::Path,
    title: &str,
) -> Result<(), String> {
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    write_tecplot(&mut std::io::BufWriter::new(f), mesh, title).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_tecplot(&mut buf, &mesh, "test").unwrap();
        let s = String::from_utf8(buf.clone()).unwrap();
        assert!(s.contains("ZONE"));

        let loaded = read_tecplot(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);
    }

    #[test]
    fn with_extra_vars() {
        let data = b"TITLE = \"Test\"\nVARIABLES = \"X\" \"Y\" \"Z\" \"P\"\nZONE T=\"Z\", N=3, E=1, F=FEPOINT, ET=TRIANGLE\n0 0 0 1.5\n1 0 0 2.5\n0 1 0 3.5\n1 2 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert!(mesh.point_data().get_array("P").is_some());
    }

    #[test]
    fn reads_nodes_elements_aliases_and_feblock() {
        let data = b"VARIABLES = \"X\" \"Y\" \"Z\" \"P\"\nZONE T=\"Z\", NODES=3, ELEMENTS=1, DATAPACKING=FEBLOCK, ET=TRIANGLE\n0 1 0\n0 0 1\n0 0 0\n10 20 30\n1 2 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert_eq!(mesh.polys.num_cells(), 1);
        assert!(mesh.point_data().get_array("P").is_some());
        assert_eq!(mesh.points.get(1), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn skips_hash_comments_like_vtk_tokenizer() {
        let data = b"# header comment\nVARIABLES = \"X\" \"Y\" \"Z\" \"P\"\nZONE T=\"Z\", N=3, E=1, F=FEPOINT, ET=TRIANGLE\n0 0 0 1\n# point comment\n1 0 0 2\n0 1 0 3\n1 2 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert_eq!(
            mesh.point_data().get_array("P").unwrap().to_f64_vec(),
            vec![1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn feblock_varlocation_adds_cell_data() {
        let data = b"VARIABLES = \"X\" \"Y\" \"Z\" \"P\" \"C\"\nZONE T=\"Z\", NODES=4, ELEMENTS=2, DATAPACKING=FEBLOCK, ET=TRIANGLE, VARLOCATION=([5]=CELLCENTERED)\n0 1 0 1\n0 0 1 1\n0 0 0 0\n10 20 30 40\n100 200\n1 2 3\n2 4 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(mesh.points.len(), 4);
        assert_eq!(mesh.polys.num_cells(), 2);
        assert_eq!(
            mesh.point_data().get_array("P").unwrap().to_f64_vec(),
            vec![10.0, 20.0, 30.0, 40.0]
        );
        assert_eq!(
            mesh.cell_data().get_array("C").unwrap().to_f64_vec(),
            vec![100.0, 200.0]
        );
        assert!(mesh.point_data().get_array("C").is_none());
    }

    #[test]
    fn point_packing_ignores_varlocation_like_vtk() {
        let data = b"VARIABLES = \"X\" \"Y\" \"Z\" \"C\" \"P\"\nZONE T=\"Z\", N=3, E=1, F=FEPOINT, ET=TRIANGLE, VARLOCATION=([4]=CELLCENTERED)\n0 0 0 100 1\n1 0 0 200 2\n0 1 0 300 3\n1 2 3\n";
        let mesh = read_tecplot(&data[..]).unwrap();
        assert_eq!(
            mesh.point_data().get_array("C").unwrap().to_f64_vec(),
            vec![100.0, 200.0, 300.0]
        );
        assert_eq!(
            mesh.point_data().get_array("P").unwrap().to_f64_vec(),
            vec![1.0, 2.0, 3.0]
        );
        assert!(mesh.cell_data().get_array("C").is_none());
    }
}
