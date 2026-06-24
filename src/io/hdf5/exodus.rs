//! Exodus II (.exo) reader and writer via HDF5.
//!
//! Exodus II is a finite element data model built on HDF5/NetCDF,
//! widely used in FEM/FEA tools (Cubit, SIERRA, Trilinos).

use crate::data::{AnyDataArray, DataArray, UnstructuredGrid};
use crate::types::{CellType, VtkError};
use std::path::Path;

use crate::types::{ExodusElementType, ExodusInfo};

/// Read an Exodus II file, returning an UnstructuredGrid + metadata.
pub fn read_exodus(path: &Path) -> Result<(UnstructuredGrid, ExodusInfo), VtkError> {
    let file = hdf5::File::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let mut info = ExodusInfo::default();

    // Read dimensions from root attributes
    if let Ok(attr) = file.attr("title") {
        info.title = attr
            .read_scalar::<hdf5::types::VarLenAscii>()
            .map(|s| s.to_string())
            .unwrap_or_default();
    }
    info.num_dim = read_dim(&file, "num_dim").unwrap_or(3);
    info.num_nodes = read_dim(&file, "num_nodes").unwrap_or(0);
    info.num_elem = read_dim(&file, "num_el_blk").unwrap_or(0);

    // Read coordinates
    let coord_x = read_f64_dataset(&file, "coordx")?;
    let coord_y = read_f64_dataset(&file, "coordy")?;
    let coord_z = if info.num_dim >= 3 {
        read_f64_dataset(&file, "coordz").unwrap_or_else(|_| vec![0.0; coord_x.len()])
    } else {
        vec![0.0; coord_x.len()]
    };

    let num_nodes = coord_x.len();
    let mut points_flat = Vec::with_capacity(num_nodes * 3);
    for i in 0..num_nodes {
        points_flat.push(coord_x[i]);
        points_flat.push(coord_y[i]);
        points_flat.push(coord_z[i]);
    }

    // Read element blocks
    let mut all_cell_types = Vec::new();
    let mut all_connectivity = Vec::new();

    let num_blk = read_dim(&file, "num_el_blk").unwrap_or(0);
    for blk in 1..=num_blk {
        let connect_name = format!("connect{blk}");
        if let Ok(ds) = file.dataset(&connect_name) {
            let shape = ds.shape();
            let num_elem = shape.get(0).copied().unwrap_or(0);
            let nodes_per_elem = shape.get(1).copied().unwrap_or(0);

            let data: Vec<i32> = ds
                .read_raw()
                .map_err(|e| VtkError::Parse(format!("read {connect_name}: {e}")))?;

            // Determine cell type from nodes per element
            let cell_type = match nodes_per_elem {
                2 => CellType::Line,
                3 => CellType::Triangle,
                4 => CellType::Tetra, // could be Quad — check dim
                6 => CellType::Wedge,
                8 => CellType::Hexahedron,
                _ => CellType::Triangle,
            };

            for e in 0..num_elem {
                all_cell_types.push(cell_type);
                let mut conn = Vec::with_capacity(nodes_per_elem);
                for n in 0..nodes_per_elem {
                    // Exodus uses 1-based indexing
                    let node_id = data[e * nodes_per_elem + n] - 1;
                    conn.push(node_id as i64);
                }
                all_connectivity.push(conn);
            }
        }
    }

    // Build UnstructuredGrid
    let points = crate::data::Points::from_vec(
        (0..num_nodes)
            .map(|i| {
                [
                    points_flat[i * 3],
                    points_flat[i * 3 + 1],
                    points_flat[i * 3 + 2],
                ]
            })
            .collect(),
    );

    let mut grid = UnstructuredGrid::new();
    grid.points = points;
    for (ct, conn) in all_cell_types.iter().zip(all_connectivity.iter()) {
        grid.insert_cell(*ct, conn);
    }

    // Read nodal variables
    if let Ok(names_ds) = file.dataset("name_nod_var") {
        if let Ok(names) = read_string_array(&names_ds) {
            info.nodal_var_names = names.clone();
            info.num_time_steps = read_dim(&file, "time_step").unwrap_or(1);

            // Read last time step's nodal vars
            let ts = info.num_time_steps;
            for (vi, name) in names.iter().enumerate() {
                let ds_name = format!("vals_nod_var{}time{}", vi + 1, ts);
                if let Ok(vals) = read_f64_dataset(&file, &ds_name) {
                    grid.point_data_mut()
                        .add_array(AnyDataArray::F64(DataArray::from_vec(name.trim(), vals, 1)));
                }
            }
        }
    }

    info.num_nodes = num_nodes;
    info.num_elem = all_cell_types.len();
    Ok((grid, info))
}

/// Write an UnstructuredGrid to Exodus II format.
pub fn write_exodus(path: &Path, grid: &UnstructuredGrid, title: &str) -> Result<(), VtkError> {
    let file = hdf5::File::create(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let num_nodes = grid.points.len();
    let num_dim = 3usize;

    // Write coordinates
    let mut cx = Vec::with_capacity(num_nodes);
    let mut cy = Vec::with_capacity(num_nodes);
    let mut cz = Vec::with_capacity(num_nodes);
    for i in 0..num_nodes {
        let p = grid.points.get(i);
        cx.push(p[0]);
        cy.push(p[1]);
        cz.push(p[2]);
    }

    write_f64_dataset(&file, "coordx", &cx)?;
    write_f64_dataset(&file, "coordy", &cy)?;
    write_f64_dataset(&file, "coordz", &cz)?;

    // Write title attribute
    file.new_attr::<hdf5::types::VarLenAscii>()
        .create("title")
        .and_then(|a| a.write_scalar(&hdf5::types::VarLenAscii::from_ascii(title).unwrap()))
        .map_err(|e| {
            VtkError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{e}"),
            ))
        })?;

    // Write dimensions
    write_dim(&file, "num_dim", num_dim)?;
    write_dim(&file, "num_nodes", num_nodes)?;

    Ok(())
}

// --- HDF5 helpers ---

fn read_dim(file: &hdf5::File, name: &str) -> Option<usize> {
    file.attr(name)
        .ok()
        .and_then(|a| a.read_scalar::<i64>().ok())
        .map(|v| v as usize)
        .or_else(|| {
            file.dataset(name)
                .ok()
                .and_then(|ds| ds.read_scalar::<i64>().ok())
                .map(|v| v as usize)
        })
}

fn read_f64_dataset(file: &hdf5::File, name: &str) -> Result<Vec<f64>, VtkError> {
    let ds = file
        .dataset(name)
        .map_err(|e| VtkError::Parse(format!("dataset '{name}': {e}")))?;
    ds.read_raw::<f64>()
        .map_err(|e| VtkError::Parse(format!("read '{name}': {e}")))
}

fn write_f64_dataset(file: &hdf5::File, name: &str, data: &[f64]) -> Result<(), VtkError> {
    file.new_dataset::<f64>()
        .shape([data.len()])
        .create(name)
        .and_then(|ds| ds.write(data))
        .map_err(|e| {
            VtkError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{e}"),
            ))
        })
}

fn write_dim(file: &hdf5::File, name: &str, value: usize) -> Result<(), VtkError> {
    file.new_attr::<i64>()
        .create(name)
        .and_then(|a| a.write_scalar(&(value as i64)))
        .map_err(|e| {
            VtkError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{e}"),
            ))
        })
}

fn read_string_array(ds: &hdf5::Dataset) -> Result<Vec<String>, VtkError> {
    ds.read_raw::<hdf5::types::VarLenAscii>()
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .map_err(|e| VtkError::Parse(format!("read strings: {e}")))
}
