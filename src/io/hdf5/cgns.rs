//! CGNS (.cgns) reader via HDF5.
//!
//! CGNS (CFD General Notation System) stores CFD data in HDF5 format
//! with a specific tree structure: Base > Zone > GridCoordinates + FlowSolution.

use crate::data::{AnyDataArray, DataArray, UnstructuredGrid};
use crate::types::{CellType, VtkError};
use std::collections::BTreeMap;
use std::path::Path;

use crate::types::CgnsInfo;

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
    let base_names = list_groups(&file)?;
    info.num_bases = base_names.len();

    let mut all_points = Vec::new();
    let mut all_cell_types = Vec::new();
    let mut all_connectivity: Vec<Vec<i64>> = Vec::new();
    let mut all_flow_data: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for base_name in &base_names {
        let base = file
            .group(base_name)
            .map_err(|e| VtkError::Parse(format!("base '{base_name}': {e}")))?;

        // Read cell/phys dimensions from base attributes
        if let Ok(attr) = base.attr("CellDimension") {
            info.cell_dim = attr.read_scalar::<i32>().unwrap_or(3) as usize;
        }
        if let Ok(attr) = base.attr("PhysicalDimension") {
            info.phys_dim = attr.read_scalar::<i32>().unwrap_or(3) as usize;
        }

        let zone_names = list_groups(&base)?;
        info.num_zones += zone_names.len();

        for zone_name in &zone_names {
            let zone = base
                .group(zone_name)
                .map_err(|e| VtkError::Parse(format!("zone '{zone_name}': {e}")))?;

            // Read coordinates
            if let Ok(grid_coords) = zone.group("GridCoordinates") {
                let cx = read_f64_ds(&grid_coords, "CoordinateX")?;
                let cy = read_f64_ds(&grid_coords, "CoordinateY")?;
                let cz = read_f64_ds(&grid_coords, "CoordinateZ")
                    .unwrap_or_else(|_| vec![0.0; cx.len()]);

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
                        &mut all_cell_types,
                        &mut all_connectivity,
                    )?;
                }

                // Read flow solution variables
                if let Ok(flow) = zone.group("FlowSolution") {
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
        2 => (CellType::Line, 2),        // BAR_2
        5 => (CellType::Triangle, 3),    // TRI_3
        7 => (CellType::Quad, 4),        // QUAD_4
        10 => (CellType::Tetra, 4),      // TETRA_4
        12 => (CellType::Pyramid, 5),    // PYRA_5
        14 => (CellType::Wedge, 6),      // PENTA_6
        17 => (CellType::Hexahedron, 8), // HEXA_8
        _ => (CellType::Triangle, 3),
    }
}

fn list_groups(loc: &hdf5::Group) -> Result<Vec<String>, VtkError> {
    loc.member_names()
        .map_err(|e| VtkError::Parse(format!("list groups: {e}")))
}

fn list_datasets(loc: &hdf5::Group) -> Vec<String> {
    loc.member_names().unwrap_or_default()
}

fn read_f64_ds(group: &hdf5::Group, name: &str) -> Result<Vec<f64>, VtkError> {
    let ds = group
        .dataset(name)
        .map_err(|e| VtkError::Parse(format!("dataset '{name}': {e}")))?;
    ds.read_raw::<f64>()
        .map_err(|e| VtkError::Parse(format!("read '{name}': {e}")))
}

fn is_cgns_node(group: &hdf5::Group, label: &str) -> bool {
    group
        .attr("label")
        .or_else(|_| group.attr("Label"))
        .and_then(|a| a.read_scalar::<hdf5::types::VarLenAscii>())
        .map(|s| s.as_str() == label)
        .unwrap_or(false)
}

fn read_cgns_section(
    section: &hdf5::Group,
    base_idx: usize,
    all_cell_types: &mut Vec<CellType>,
    all_connectivity: &mut Vec<Vec<i64>>,
) -> Result<(), VtkError> {
    let conn_ds = section
        .dataset("ElementConnectivity")
        .map_err(|e| VtkError::Parse(format!("ElementConnectivity: {e}")))?;
    let conn: Vec<i32> = conn_ds
        .read_raw()
        .map_err(|e| VtkError::Parse(format!("connectivity: {e}")))?;
    let etype = section
        .attr("ElementType")
        .and_then(|a| a.read_scalar::<i32>())
        .or_else(|_| {
            section
                .dataset("ElementType")
                .and_then(|ds| ds.read_scalar::<i32>())
        })
        .unwrap_or(5);

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
