//! Geodesic Voronoi partition of a mesh's vertices.
//!
//! Single implementation in [`crate::filters::mesh::mesh_geodesic_voronoi`]:
//! edge-weighted Dijkstra from every seed (a true geodesic partition, unlike the
//! Euclidean `mesh_voronoi_partition`). It labels points with "VoronoiRegion"
//! (this module used to call the array "GeodesicRegion") and also emits the
//! distance as "VoronoiDist".

pub use crate::filters::mesh::mesh_geodesic_voronoi::geodesic_voronoi;
