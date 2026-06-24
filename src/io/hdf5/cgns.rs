//! CGNS (.cgns) reader via HDF5.
//!
//! CGNS (CFD General Notation System) stores CFD data in HDF5 format
//! with a specific tree structure: Base > Zone > GridCoordinates + FlowSolution.

use crate::data::{AnyDataArray, DataArray, UnstructuredGrid};
use crate::types::{CellType, VtkError};
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

                // Read element connectivity if unstructured
                if let Ok(elems) = zone.group("Elements") {
                    if let Ok(conn_ds) = elems.dataset("ElementConnectivity") {
                        let conn: Vec<i32> = conn_ds
                            .read_raw()
                            .map_err(|e| VtkError::Parse(format!("connectivity: {e}")))?;
                        // Read element type
                        let etype = elems
                            .attr("ElementType")
                            .and_then(|a| a.read_scalar::<i32>())
                            .unwrap_or(5); // default TRI_3

                        let (cell_type, npn) = cgns_element_type(etype);
                        for chunk in conn.chunks_exact(npn) {
                            let cell: Vec<i64> = chunk
                                .iter()
                                .map(|&v| (v - 1) as i64 + base_idx as i64) // CGNS 1-based
                                .collect();
                            all_cell_types.push(cell_type);
                            all_connectivity.push(cell);
                        }
                    }
                }

                // Read flow solution variables
                if let Ok(flow) = zone.group("FlowSolution") {
                    let var_names = list_datasets(&flow);
                    for vname in &var_names {
                        if let Ok(vals) = read_f64_ds(&flow, vname) {
                            // Will be added as point data below
                            let _ = vals; // placeholder
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
        grid.insert_cell(*ct, conn);
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
