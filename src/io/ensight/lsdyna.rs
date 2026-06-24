use std::path::Path;

use crate::data::{CellArray, Points, PolyData, UnstructuredGrid};
use crate::types::{CellType, VtkError};

/// Reader for LS-DYNA keyword input files (.k, .key, .dyn).
///
/// Supports reading `*NODE` and `*ELEMENT_SHELL` / `*ELEMENT_SOLID` sections
/// from ASCII keyword format files. This covers the most common mesh data.
pub struct LsDynaReader;

impl LsDynaReader {
    /// Read an LS-DYNA keyword file and return a PolyData (for shells)
    /// or UnstructuredGrid (for solids).
    pub fn read_poly_data(path: &Path) -> Result<PolyData, VtkError> {
        let content = std::fs::read_to_string(path)?;
        parse_keyword_poly_data(&content)
    }

    /// Read LS-DYNA keyword content from a string.
    pub fn read_poly_data_from_str(content: &str) -> Result<PolyData, VtkError> {
        parse_keyword_poly_data(content)
    }

    /// Read an LS-DYNA keyword file and return an UnstructuredGrid.
    pub fn read_unstructured(path: &Path) -> Result<UnstructuredGrid, VtkError> {
        let content = std::fs::read_to_string(path)?;
        parse_keyword_unstructured(&content)
    }
}

fn parse_keyword_poly_data(content: &str) -> Result<PolyData, VtkError> {
    let (nodes, shell_elements) = parse_keyword_content(content)?;

    let mut pd = PolyData::new();

    // Build node ID to index mapping
    let mut id_to_idx = std::collections::HashMap::new();
    for (i, (nid, pos)) in nodes.iter().enumerate() {
        id_to_idx.insert(*nid, i);
        pd.points.push(*pos);
    }

    // Build polys from shell elements
    for (_eid, node_ids) in &shell_elements {
        let indices: Vec<i64> = node_ids
            .iter()
            .filter_map(|nid| id_to_idx.get(nid).map(|&idx| idx as i64))
            .collect();
        if indices.len() >= 3 {
            // Check for degenerate quads (repeated last node)
            if indices.len() == 4 && indices[2] == indices[3] {
                pd.polys.push_cell(&indices[..3]);
            } else {
                pd.polys.push_cell(&indices);
            }
        }
    }

    Ok(pd)
}

fn parse_keyword_unstructured(content: &str) -> Result<UnstructuredGrid, VtkError> {
    let (nodes, shell_elements) = parse_keyword_content(content)?;

    let mut points = Points::new();
    let mut cells = CellArray::new();
    let mut cell_types = Vec::new();

    let mut id_to_idx = std::collections::HashMap::new();
    for (i, (nid, pos)) in nodes.iter().enumerate() {
        id_to_idx.insert(*nid, i);
        points.push(*pos);
    }

    for (_eid, node_ids) in &shell_elements {
        let indices: Vec<i64> = node_ids
            .iter()
            .filter_map(|nid| id_to_idx.get(nid).map(|&idx| idx as i64))
            .collect();
        if indices.len() == 4 && indices[2] != indices[3] {
            cells.push_cell(&indices);
            cell_types.push(CellType::Quad);
        } else if indices.len() >= 3 {
            cells.push_cell(&indices[..3]);
            cell_types.push(CellType::Triangle);
        }
    }

    let mut ug = UnstructuredGrid::new();
    ug.points = points;
    for (i, cell) in cells.iter().enumerate() {
        ug.push_cell(cell_types[i], cell);
    }
    Ok(ug)
}

/// Parse *NODE and *ELEMENT_SHELL sections from keyword content.
/// Returns (nodes: [(id, [x,y,z])], elements: [(id, [n1,n2,n3,n4])]).
fn parse_keyword_content(
    content: &str,
) -> Result<(Vec<(i64, [f64; 3])>, Vec<(i64, Vec<i64>)>), VtkError> {
    let mut nodes = Vec::new();
    let mut elements = Vec::new();
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('$') || trimmed.is_empty() {
            continue;
        }

        // Keyword detection
        if trimmed.starts_with('*') {
            let kw = trimmed.to_uppercase();
            if kw.starts_with("*NODE") && !kw.contains("SET") {
                section = Section::Node;
            } else if kw.starts_with("*ELEMENT_SHELL") {
                section = Section::ElementShell;
            } else if kw.starts_with("*ELEMENT_SOLID") {
                section = Section::ElementSolid;
            } else {
                section = Section::None;
            }
            continue;
        }

        match section {
            Section::Node => {
                if let Some(node) = parse_node_line(trimmed) {
                    nodes.push(node);
                }
            }
            Section::ElementShell => {
                if let Some(elem) = parse_element_shell_line(trimmed) {
                    elements.push(elem);
                }
            }
            Section::ElementSolid => {
                // Solid elements have 8 nodes - skip for PolyData but include for unstructured
                if let Some(elem) = parse_element_solid_line(trimmed) {
                    elements.push(elem);
                }
            }
            Section::None => {}
        }
    }

    Ok((nodes, elements))
}

#[derive(Clone, Copy)]
enum Section {
    None,
    Node,
    ElementShell,
    ElementSolid,
}

/// Parse a *NODE data line: "nid, x, y, z" or fixed-width format.
fn parse_node_line(line: &str) -> Option<(i64, [f64; 3])> {
    // Try comma-separated first
    if line.contains(',') {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let nid: i64 = parts[0].trim().parse().ok()?;
            let x: f64 = parts[1].trim().parse().ok()?;
            let y: f64 = parts[2].trim().parse().ok()?;
            let z: f64 = parts[3].trim().parse().ok()?;
            return Some((nid, [x, y, z]));
        }
    }

    // Fixed-width format: 8 chars for nid, 16 chars each for x, y, z
    if line.len() >= 56 {
        let nid: i64 = line[0..8].trim().parse().ok()?;
        let x: f64 = line[8..24].trim().parse().ok()?;
        let y: f64 = line[24..40].trim().parse().ok()?;
        let z: f64 = line[40..56].trim().parse().ok()?;
        return Some((nid, [x, y, z]));
    }

    // Space-separated fallback
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 4 {
        let nid: i64 = parts[0].parse().ok()?;
        let x: f64 = parts[1].parse().ok()?;
        let y: f64 = parts[2].parse().ok()?;
        let z: f64 = parts[3].parse().ok()?;
        return Some((nid, [x, y, z]));
    }

    None
}

/// Parse a *ELEMENT_SHELL data line: "eid, pid, n1, n2, n3, n4".
fn parse_element_shell_line(line: &str) -> Option<(i64, Vec<i64>)> {
    let parts: Vec<i64> = if line.contains(',') {
        line.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    } else {
        line.split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    if parts.len() >= 5 {
        let eid = parts[0];
        // parts[1] is PID, parts[2..] are node IDs
        let node_ids: Vec<i64> = parts[2..].to_vec();
        Some((eid, node_ids))
    } else {
        None
    }
}

/// Parse a *ELEMENT_SOLID data line: "eid, pid, n1..n8".
fn parse_element_solid_line(line: &str) -> Option<(i64, Vec<i64>)> {
    parse_element_shell_line(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_keyword() {
        let content = r#"
$# LS-DYNA keyword file
*KEYWORD
*NODE
       1       0.0       0.0       0.0
       2       1.0       0.0       0.0
       3       0.5       1.0       0.0
*ELEMENT_SHELL
       1       1       1       2       3       3
*END
"#;
        let pd = LsDynaReader::read_poly_data_from_str(content).unwrap();
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn parse_comma_separated() {
        let content = "*NODE\n1, 0.0, 0.0, 0.0\n2, 1.0, 0.0, 0.0\n3, 0.0, 1.0, 0.0\n*ELEMENT_SHELL\n1, 1, 1, 2, 3, 3\n*END\n";
        let pd = LsDynaReader::read_poly_data_from_str(content).unwrap();
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn parse_quad_element() {
        let content =
            "*NODE\n1 0 0 0\n2 1 0 0\n3 1 1 0\n4 0 1 0\n*ELEMENT_SHELL\n1 1 1 2 3 4\n*END\n";
        let pd = LsDynaReader::read_poly_data_from_str(content).unwrap();
        assert_eq!(pd.points.len(), 4);
        assert_eq!(pd.polys.num_cells(), 1);
        assert_eq!(pd.polys.cell(0).len(), 4);
    }
}
