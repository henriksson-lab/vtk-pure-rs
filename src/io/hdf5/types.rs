//! Common types for HDF5-based I/O (always available, no feature gate).

/// Exodus element block type.
#[derive(Debug, Clone)]
pub enum ExodusElementType {
    Tri3,
    Quad4,
    Tet4,
    Hex8,
    Wedge6,
    Pyramid5,
    Shell4,
    Bar2,
}

impl ExodusElementType {
    /// Number of nodes per element.
    pub fn nodes_per_element(&self) -> usize {
        match self {
            Self::Bar2 => 2,
            Self::Tri3 => 3,
            Self::Quad4 | Self::Shell4 => 4,
            Self::Tet4 => 4,
            Self::Pyramid5 => 5,
            Self::Wedge6 => 6,
            Self::Hex8 => 8,
        }
    }

    /// Map from Exodus type string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRIANGLE" | "TRI3" | "TRI" => Some(Self::Tri3),
            "QUAD4" | "QUAD" => Some(Self::Quad4),
            "SHELL4" | "SHELL" => Some(Self::Shell4),
            "TETRA4" | "TETRA" | "TET4" | "TET" => Some(Self::Tet4),
            "HEX8" | "HEX" => Some(Self::Hex8),
            "WEDGE6" | "WEDGE" => Some(Self::Wedge6),
            "PYRAMID5" | "PYRAMID" => Some(Self::Pyramid5),
            "BAR2" | "BAR" | "BEAM" | "TRUSS" | "EDGE2" | "EDGE" => Some(Self::Bar2),
            _ => None,
        }
    }
}

/// Exodus file metadata.
#[derive(Debug, Clone, Default)]
pub struct ExodusInfo {
    pub title: String,
    pub num_dim: usize,
    pub num_nodes: usize,
    pub num_elem: usize,
    pub num_elem_blocks: usize,
    pub num_node_sets: usize,
    pub num_side_sets: usize,
    pub num_time_steps: usize,
    pub nodal_var_names: Vec<String>,
    pub elem_var_names: Vec<String>,
    pub global_var_names: Vec<String>,
}

/// CGNS zone type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgnsZoneType {
    Structured,
    Unstructured,
}

/// CGNS file metadata.
#[derive(Debug, Clone, Default)]
pub struct CgnsInfo {
    pub num_bases: usize,
    pub num_zones: usize,
    pub cell_dim: usize,
    pub phys_dim: usize,
}

/// NetCDF variable info.
#[derive(Debug, Clone)]
pub struct NetcdfVarInfo {
    pub name: String,
    pub dimensions: Vec<String>,
    pub shape: Vec<usize>,
    pub dtype: String,
}

/// AMR level info.
#[derive(Debug, Clone)]
pub struct AmrLevelInfo {
    pub level: usize,
    pub num_boxes: usize,
    pub refinement_ratio: usize,
    pub dx: [f64; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exodus_element_types() {
        assert_eq!(ExodusElementType::Hex8.nodes_per_element(), 8);
        assert_eq!(ExodusElementType::Tet4.nodes_per_element(), 4);
        assert!(ExodusElementType::from_str("HEX8").is_some());
        assert!(matches!(
            ExodusElementType::from_str("SHELL4"),
            Some(ExodusElementType::Shell4)
        ));
        assert!(matches!(
            ExodusElementType::from_str("TRIANGLE"),
            Some(ExodusElementType::Tri3)
        ));
        assert!(matches!(
            ExodusElementType::from_str("EDGE2"),
            Some(ExodusElementType::Bar2)
        ));
        assert!(ExodusElementType::from_str("UNKNOWN").is_none());
    }

    #[test]
    fn exodus_info_default() {
        let info = ExodusInfo::default();
        assert_eq!(info.num_nodes, 0);
    }

    #[test]
    fn cgns_zone_type() {
        assert_ne!(CgnsZoneType::Structured, CgnsZoneType::Unstructured);
    }
}
