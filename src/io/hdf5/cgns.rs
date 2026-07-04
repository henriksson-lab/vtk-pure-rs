//! CGNS (.cgns) reader via HDF5.
//!
//! CGNS (CFD General Notation System) stores CFD data in HDF5 format
//! with a specific tree structure: Base > Zone > GridCoordinates + FlowSolution.

use crate::data::{AnyDataArray, DataArray, UnstructuredGrid};
use crate::types::{CellType, VtkError};
use std::collections::BTreeMap;
use std::path::Path;

use super::types::CgnsInfo;

/// Read a CGNS file, returning an UnstructuredGrid + metadata.
pub fn read_cgns(path: &Path) -> Result<(UnstructuredGrid, CgnsInfo), VtkError> {
    let file = hdf5::File::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let mut info = CgnsInfo::default();

    // CGNS HDF5 structure: / > CGNSBase > Zone > GridCoordinates/{CoordinateX,Y,Z}
    let base_names = cgns_group_names(&file, "CGNSBase_t")?;
    info.num_bases = base_names.len();

    let mut all_points = Vec::new();
    let mut all_cell_types = Vec::new();
    let mut all_connectivity: Vec<Vec<i64>> = Vec::new();
    let mut all_flow_data: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for base_name in &base_names {
        let base = file
            .group(base_name)
            .map_err(|e| VtkError::Parse(format!("base '{base_name}': {e}")))?;

        // Read cell/phys dimensions from base attributes or the CGNSBase_t node payload.
        if let Ok(dims) = read_i32_node_payload(&base) {
            if dims.len() >= 2 {
                info.cell_dim = dims[0] as usize;
                info.phys_dim = dims[1] as usize;
            }
        }
        if let Ok(attr) = base.attr("CellDimension") {
            info.cell_dim = attr.read_scalar::<i32>().unwrap_or(3) as usize;
        }
        if let Ok(attr) = base.attr("PhysicalDimension") {
            info.phys_dim = attr.read_scalar::<i32>().unwrap_or(3) as usize;
        }

        let zone_names = cgns_group_names(&base, "Zone_t")?;
        info.num_zones += zone_names.len();

        for zone_name in &zone_names {
            let zone = base
                .group(zone_name)
                .map_err(|e| VtkError::Parse(format!("zone '{zone_name}': {e}")))?;

            // Read coordinates
            if let Some(grid_coords_name) =
                cgns_child_group_name(&zone, "GridCoordinates_t", "GridCoordinates")?
            {
                let grid_coords = zone.group(&grid_coords_name).map_err(|e| {
                    VtkError::Parse(format!("grid coordinates '{grid_coords_name}': {e}"))
                })?;
                let cx = read_f64_ds(&grid_coords, "CoordinateX")?;
                let cy = read_f64_ds(&grid_coords, "CoordinateY")?;
                let cz = read_f64_ds(&grid_coords, "CoordinateZ")
                    .unwrap_or_else(|_| vec![0.0; cx.len()]);
                if cy.len() != cx.len() || cz.len() != cx.len() {
                    return Err(VtkError::Parse(format!(
                        "coordinate array length mismatch in '{grid_coords_name}'"
                    )));
                }

                let base_idx = all_points.len() / 3;
                for i in 0..cx.len() {
                    all_points.push(cx[i]);
                    all_points.push(cy[i]);
                    all_points.push(cz[i]);
                }

                for section_name in list_groups(&zone)? {
                    let Ok(section) = zone.group(&section_name) else {
                        continue;
                    };
                    if !is_cgns_node(&section, "Elements_t")
                        && section.dataset("ElementConnectivity").is_err()
                    {
                        continue;
                    }
                    read_cgns_section(
                        &section,
                        base_idx,
                        info.cell_dim,
                        &mut all_cell_types,
                        &mut all_connectivity,
                    )?;
                }

                // Read flow solution variables
                for flow_name in flow_solution_group_names(&zone)? {
                    if let Ok(flow) = zone.group(&flow_name) {
                        let var_names = list_datasets(&flow);
                        for vname in &var_names {
                            if let Ok(vals) = read_f64_ds(&flow, vname) {
                                if vals.len() == cx.len() {
                                    all_flow_data.entry(vname.clone()).or_default().extend(vals);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let num_nodes = all_points.len() / 3;
    let points = crate::data::Points::from_vec(
        (0..num_nodes)
            .map(|i| {
                [
                    all_points[i * 3],
                    all_points[i * 3 + 1],
                    all_points[i * 3 + 2],
                ]
            })
            .collect(),
    );

    let mut grid = UnstructuredGrid::new();
    grid.points = points;
    for (ct, conn) in all_cell_types.iter().zip(all_connectivity.iter()) {
        grid.push_cell(*ct, conn);
    }
    for (name, vals) in all_flow_data {
        if vals.len() == num_nodes {
            grid.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(name, vals, 1)));
        }
    }

    Ok((grid, info))
}

fn cgns_element_type(etype: i32) -> (CellType, usize) {
    match etype {
        2 => (CellType::Vertex, cgns_num_points(etype)), // NODE
        3 => (CellType::Line, cgns_num_points(etype)),   // BAR_2
        5 => (CellType::Triangle, cgns_num_points(etype)), // TRI_3
        7 => (CellType::Quad, cgns_num_points(etype)),   // QUAD_4
        10 => (CellType::Tetra, cgns_num_points(etype)), // TETRA_4
        12 => (CellType::Pyramid, cgns_num_points(etype)), // PYRA_5
        14 => (CellType::Wedge, cgns_num_points(etype)), // PENTA_6
        17 => (CellType::Hexahedron, cgns_num_points(etype)), // HEXA_8
        _ => (CellType::Empty, 0),
    }
}

fn cgns_num_points(etype: i32) -> usize {
    match etype {
        2 => 1,    // NODE
        3 => 2,    // BAR_2
        4 => 3,    // BAR_3
        5 => 3,    // TRI_3
        6 => 6,    // TRI_6
        7 => 4,    // QUAD_4
        8 => 8,    // QUAD_8
        9 => 9,    // QUAD_9
        10 => 4,   // TETRA_4
        11 => 10,  // TETRA_10
        12 => 5,   // PYRA_5
        13 => 14,  // PYRA_14
        14 => 6,   // PENTA_6
        15 => 15,  // PENTA_15
        16 => 18,  // PENTA_18
        17 => 8,   // HEXA_8
        18 => 20,  // HEXA_20
        19 => 27,  // HEXA_27
        21 => 13,  // PYRA_13
        24 => 4,   // BAR_4
        25 => 9,   // TRI_9
        26 => 10,  // TRI_10
        27 => 12,  // QUAD_12
        28 => 16,  // QUAD_16
        29 => 16,  // TETRA_16
        30 => 20,  // TETRA_20
        31 => 21,  // PYRA_21
        32 => 29,  // PYRA_29
        33 => 30,  // PYRA_30
        34 => 24,  // PENTA_24
        35 => 38,  // PENTA_38
        36 => 40,  // PENTA_40
        37 => 32,  // HEXA_32
        38 => 56,  // HEXA_56
        39 => 64,  // HEXA_64
        40 => 5,   // BAR_5
        41 => 12,  // TRI_12
        42 => 15,  // TRI_15
        43 => 16,  // QUAD_P4_16
        44 => 25,  // QUAD_25
        45 => 22,  // TETRA_22
        46 => 34,  // TETRA_34
        47 => 35,  // TETRA_35
        48 => 29,  // PYRA_P4_29
        49 => 50,  // PYRA_50
        50 => 55,  // PYRA_55
        51 => 33,  // PENTA_33
        52 => 66,  // PENTA_66
        53 => 75,  // PENTA_75
        54 => 44,  // HEXA_44
        55 => 98,  // HEXA_98
        56 => 125, // HEXA_125
        _ => 0,
    }
}

fn cgns_element_dimension(etype: i32) -> Option<usize> {
    match etype {
        2 => Some(0),               // NODE
        3 | 4 | 24 | 40 => Some(1), // BAR_*
        5..=9 | 22 | 25..=28 | 41..=44 => Some(2),
        10..=19 | 21 | 23 | 29..=39 | 45..=56 => Some(3),
        _ => None,
    }
}

fn list_groups(loc: &hdf5::Group) -> Result<Vec<String>, VtkError> {
    let mut groups = Vec::new();
    for name in loc
        .member_names()
        .map_err(|e| VtkError::Parse(format!("list groups: {e}")))?
    {
        if name.starts_with(' ') {
            continue;
        }
        if loc.group(&name).is_ok() {
            groups.push(name);
        }
    }
    Ok(groups)
}

fn cgns_child_group_name(
    loc: &hdf5::Group,
    label: &str,
    fallback_name: &str,
) -> Result<Option<String>, VtkError> {
    let groups = list_groups(loc)?;
    if let Some(name) = groups.iter().find(|name| {
        loc.group(name)
            .map(|group| is_cgns_node(&group, label))
            .unwrap_or(false)
    }) {
        return Ok(Some(name.clone()));
    }
    if loc.group(fallback_name).is_ok() {
        Ok(Some(fallback_name.to_string()))
    } else {
        Ok(None)
    }
}

fn cgns_group_names(loc: &hdf5::Group, label: &str) -> Result<Vec<String>, VtkError> {
    let groups = list_groups(loc)?;
    let labeled: Vec<String> = groups
        .iter()
        .filter(|name| {
            loc.group(name)
                .map(|group| is_cgns_node(&group, label))
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    if labeled.is_empty() {
        Ok(groups)
    } else {
        Ok(labeled)
    }
}

fn flow_solution_group_names(loc: &hdf5::Group) -> Result<Vec<String>, VtkError> {
    Ok(list_groups(loc)?
        .into_iter()
        .filter(|name| {
            name.starts_with("FlowSolution")
                || loc
                    .group(name)
                    .map(|group| is_cgns_node(&group, "FlowSolution_t"))
                    .unwrap_or(false)
        })
        .collect())
}

fn list_datasets(loc: &hdf5::Group) -> Vec<String> {
    loc.member_names()
        .unwrap_or_default()
        .into_iter()
        .filter(|name| {
            loc.dataset(name).is_ok()
                || loc
                    .group(name)
                    .map(|group| {
                        is_cgns_node(&group, "DataArray_t")
                            || group.dataset(" data").is_ok()
                            || group.dataset("data").is_ok()
                    })
                    .unwrap_or(false)
        })
        .collect()
}

fn read_f64_ds(group: &hdf5::Group, name: &str) -> Result<Vec<f64>, VtkError> {
    if let Ok(ds) = group.dataset(name) {
        return read_dataset_as_f64(&ds, name);
    }
    let node = group
        .group(name)
        .map_err(|e| VtkError::Parse(format!("dataset '{name}': {e}")))?;
    read_f64_node_payload(&node)
}

fn read_f64_node_payload(group: &hdf5::Group) -> Result<Vec<f64>, VtkError> {
    for name in [" data", "data"] {
        if let Ok(ds) = group.dataset(name) {
            return read_dataset_as_f64(&ds, group.name().as_str());
        }
    }
    Err(VtkError::Parse(format!(
        "node '{}' has no data payload",
        group.name()
    )))
}

fn read_dataset_as_f64(ds: &hdf5::Dataset, name: &str) -> Result<Vec<f64>, VtkError> {
    ds.read_raw::<f64>()
        .or_else(|_| {
            ds.read_raw::<f32>()
                .map(|values| values.into_iter().map(f64::from).collect())
        })
        .or_else(|_| {
            ds.read_raw::<i32>()
                .map(|values| values.into_iter().map(f64::from).collect())
        })
        .or_else(|_| {
            ds.read_raw::<i64>()
                .map(|values| values.into_iter().map(|v| v as f64).collect())
        })
        .map_err(|e| VtkError::Parse(format!("read '{name}': {e}")))
}

fn read_i32_node(group: &hdf5::Group, name: &str) -> Result<Vec<i32>, VtkError> {
    if let Ok(ds) = group.dataset(name) {
        return read_dataset_as_i32(&ds, name);
    }
    let node = group
        .group(name)
        .map_err(|e| VtkError::Parse(format!("dataset '{name}': {e}")))?;
    read_i32_node_payload(&node)
}

fn read_i32_node_payload(group: &hdf5::Group) -> Result<Vec<i32>, VtkError> {
    for name in [" data", "data"] {
        if let Ok(ds) = group.dataset(name) {
            return read_dataset_as_i32(&ds, group.name().as_str());
        }
    }
    Err(VtkError::Parse(format!(
        "node '{}' has no data payload",
        group.name()
    )))
}

fn read_dataset_as_i32(ds: &hdf5::Dataset, name: &str) -> Result<Vec<i32>, VtkError> {
    ds.read_raw::<i32>()
        .or_else(|_| {
            ds.read_raw::<i64>()
                .map(|values| values.into_iter().map(|v| v as i32).collect())
        })
        .map_err(|e| VtkError::Parse(format!("read '{name}': {e}")))
}

fn is_cgns_node(group: &hdf5::Group, label: &str) -> bool {
    read_cgns_string_attr(group, "label")
        .or_else(|| read_cgns_string_attr(group, "Label"))
        .map(|s| s == label)
        .unwrap_or(false)
}

fn read_cgns_string_attr(group: &hdf5::Group, name: &str) -> Option<String> {
    let attr = group.attr(name).ok()?;
    if let Ok(value) = attr.read_scalar::<hdf5::types::VarLenAscii>() {
        return Some(value.as_str().to_string());
    }
    if let Ok(value) = attr.read_scalar::<hdf5::types::FixedAscii<33>>() {
        return Some(value.as_str().to_string());
    }
    attr.read_raw::<u8>().ok().and_then(|bytes| {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..end]).ok().map(str::to_string)
    })
}

fn read_cgns_section(
    section: &hdf5::Group,
    base_idx: usize,
    cell_dim: usize,
    all_cell_types: &mut Vec<CellType>,
    all_connectivity: &mut Vec<Vec<i64>>,
) -> Result<(), VtkError> {
    let conn = read_i32_node(section, "ElementConnectivity")
        .map_err(|e| VtkError::Parse(format!("connectivity: {e}")))?;
    let etype = read_section_element_type(section)?;

    if etype == 20 {
        return read_mixed_cgns_section(
            &conn,
            base_idx,
            cell_dim,
            all_cell_types,
            all_connectivity,
        );
    }

    if cell_dim > 0 && cgns_element_dimension(etype) == Some(cell_dim - 1) {
        return Ok(());
    }

    let (cell_type, npn) = cgns_element_type(etype);
    if npn == 0 {
        return Ok(());
    }
    for chunk in conn.chunks_exact(npn) {
        let cell: Vec<i64> = chunk
            .iter()
            .map(|&v| (v - 1) as i64 + base_idx as i64)
            .collect();
        all_cell_types.push(cell_type);
        all_connectivity.push(cell);
    }
    Ok(())
}

fn read_mixed_cgns_section(
    conn: &[i32],
    base_idx: usize,
    cell_dim: usize,
    all_cell_types: &mut Vec<CellType>,
    all_connectivity: &mut Vec<Vec<i64>>,
) -> Result<(), VtkError> {
    let mut pos = 0usize;
    while pos < conn.len() {
        let etype = conn[pos];
        pos += 1;

        let npn = cgns_num_points(etype);
        if npn == 0 {
            return Err(VtkError::Parse(format!(
                "unsupported MIXED CGNS element type {etype}"
            )));
        }
        if pos + npn > conn.len() {
            return Err(VtkError::Parse(format!(
                "truncated MIXED CGNS element type {etype}"
            )));
        }

        let skip_boundary = cell_dim > 0 && cgns_element_dimension(etype) == Some(cell_dim - 1);
        let (cell_type, supported_npn) = cgns_element_type(etype);
        if !skip_boundary && supported_npn == npn {
            let cell: Vec<i64> = conn[pos..pos + npn]
                .iter()
                .map(|&v| (v - 1) as i64 + base_idx as i64)
                .collect();
            all_cell_types.push(cell_type);
            all_connectivity.push(cell);
        }

        pos += npn;
    }
    Ok(())
}

fn read_section_element_type(section: &hdf5::Group) -> Result<i32, VtkError> {
    if let Ok(attr) = section.attr("ElementType") {
        return attr
            .read_scalar::<i32>()
            .map_err(|e| VtkError::Parse(format!("ElementType attribute: {e}")));
    }
    if let Ok(ds) = section.dataset("ElementType") {
        return ds
            .read_scalar::<i32>()
            .map_err(|e| VtkError::Parse(format!("ElementType dataset: {e}")));
    }
    read_i32_node_payload(section).and_then(|data| {
        data.first().copied().ok_or_else(|| {
            VtkError::Parse(format!("empty Elements_t payload '{}'", section.name()))
        })
    })
}
