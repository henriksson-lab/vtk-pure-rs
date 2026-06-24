//! Discontinuous Galerkin / high-order cell grid representation.
//!
//! `CellGrid` is VTK 9.3+'s new data model for discontinuous fields on
//! high-order cells. Unlike `UnstructuredGrid` where cells share nodes,
//! in a `CellGrid` each cell has its own set of DOF (degrees of freedom)
//! nodes, enabling discontinuous field representations.
//!
//! # Design
//!
//! - Each cell type has its own `CellSpec` defining the reference element
//! - DOFs are stored per-cell, not shared between cells
//! - Fields can be discontinuous across cell boundaries
//! - Supports Lagrange and Bezier basis functions via `crate::types::higher_order`

use crate::data::FieldData;
use crate::types::{BoundingBox, CellType};
use std::collections::HashMap;

/// Reference element specification for a cell type.
#[derive(Debug, Clone)]
pub struct CellSpec {
    /// VTK cell type.
    pub cell_type: CellType,
    /// Polynomial order of the basis.
    pub order: usize,
    /// Number of DOF nodes per cell.
    pub dofs_per_cell: usize,
    /// Parametric coordinates of DOF nodes in reference space.
    /// Each entry is [xi, eta, zeta] in [0,1]^dim.
    pub reference_points: Vec<[f64; 3]>,
    /// Topological dimension (1=curve, 2=surface, 3=volume).
    pub dimension: usize,
}

impl CellSpec {
    /// Validate that the specification is internally consistent.
    pub fn validate(&self) -> bool {
        self.order > 0
            && matches!(
                self.cell_type,
                CellType::LagrangeCurve
                    | CellType::LagrangeTriangle
                    | CellType::LagrangeQuadrilateral
                    | CellType::LagrangeTetrahedron
                    | CellType::LagrangeHexahedron
            )
            && self.dimension == self.cell_type.dimension()
            && self.dofs_per_cell > 0
            && self.reference_points.len() == self.dofs_per_cell
            && self
                .reference_points
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite()))
    }

    /// Create a Lagrange triangle spec of given order.
    pub fn lagrange_triangle(order: usize) -> Self {
        assert!(order > 0, "Lagrange cell order must be greater than zero");
        let n = (order + 1) * (order + 2) / 2;
        let mut ref_pts = Vec::with_capacity(n);
        for j in 0..=order {
            for i in 0..=(order - j) {
                ref_pts.push([i as f64 / order as f64, j as f64 / order as f64, 0.0]);
            }
        }
        Self {
            cell_type: CellType::LagrangeTriangle,
            order,
            dofs_per_cell: n,
            reference_points: ref_pts,
            dimension: 2,
        }
    }

    /// Create a Lagrange quadrilateral spec of given order.
    pub fn lagrange_quad(order: usize) -> Self {
        assert!(order > 0, "Lagrange cell order must be greater than zero");
        let n = (order + 1) * (order + 1);
        let mut ref_pts = Vec::with_capacity(n);
        for j in 0..=order {
            for i in 0..=order {
                ref_pts.push([i as f64 / order as f64, j as f64 / order as f64, 0.0]);
            }
        }
        Self {
            cell_type: CellType::LagrangeQuadrilateral,
            order,
            dofs_per_cell: n,
            reference_points: ref_pts,
            dimension: 2,
        }
    }

    /// Create a Lagrange tetrahedron spec of given order.
    pub fn lagrange_tet(order: usize) -> Self {
        assert!(order > 0, "Lagrange cell order must be greater than zero");
        let mut ref_pts = Vec::new();
        for k in 0..=order {
            for j in 0..=(order - k) {
                for i in 0..=(order - j - k) {
                    ref_pts.push([
                        i as f64 / order as f64,
                        j as f64 / order as f64,
                        k as f64 / order as f64,
                    ]);
                }
            }
        }
        Self {
            cell_type: CellType::LagrangeTetrahedron,
            order,
            dofs_per_cell: ref_pts.len(),
            dimension: 3,
            reference_points: ref_pts,
        }
    }

    /// Create a Lagrange hexahedron spec of given order.
    pub fn lagrange_hex(order: usize) -> Self {
        assert!(order > 0, "Lagrange cell order must be greater than zero");
        let n = (order + 1).pow(3);
        let mut ref_pts = Vec::with_capacity(n);
        for k in 0..=order {
            for j in 0..=order {
                for i in 0..=order {
                    ref_pts.push([
                        i as f64 / order as f64,
                        j as f64 / order as f64,
                        k as f64 / order as f64,
                    ]);
                }
            }
        }
        Self {
            cell_type: CellType::LagrangeHexahedron,
            order,
            dofs_per_cell: n,
            reference_points: ref_pts,
            dimension: 3,
        }
    }

    /// Create a Lagrange curve spec of given order.
    pub fn lagrange_curve(order: usize) -> Self {
        assert!(order > 0, "Lagrange cell order must be greater than zero");
        let n = order + 1;
        let ref_pts: Vec<[f64; 3]> = (0..n)
            .map(|i| [i as f64 / order as f64, 0.0, 0.0])
            .collect();
        Self {
            cell_type: CellType::LagrangeCurve,
            order,
            dofs_per_cell: n,
            reference_points: ref_pts,
            dimension: 1,
        }
    }

    /// Evaluate Lagrange basis functions at a parametric point.
    /// Returns `dofs_per_cell` weights that sum to 1.
    pub fn evaluate_basis(&self, xi: [f64; 3]) -> Vec<f64> {
        match self.dimension {
            1 => crate::types::higher_order::lagrange_1d(self.order, xi[0]),
            2 => self.evaluate_basis_2d(xi[0], xi[1]),
            3 => self.evaluate_basis_3d(xi[0], xi[1], xi[2]),
            _ => vec![1.0],
        }
    }

    fn evaluate_basis_2d(&self, xi: f64, eta: f64) -> Vec<f64> {
        // Tensor-product for quads, special handling for triangles
        if self.cell_type == CellType::LagrangeQuadrilateral {
            let bx = crate::types::higher_order::lagrange_1d(self.order, xi);
            let by = crate::types::higher_order::lagrange_1d(self.order, eta);
            let mut result = Vec::with_capacity(self.dofs_per_cell);
            for j in 0..=self.order {
                for i in 0..=self.order {
                    result.push(bx[i] * by[j]);
                }
            }
            result
        } else {
            lagrange_simplex_basis(self.order, &self.reference_points, [xi, eta, 0.0], 2)
        }
    }

    fn evaluate_basis_3d(&self, xi: f64, eta: f64, zeta: f64) -> Vec<f64> {
        if self.cell_type == CellType::LagrangeHexahedron {
            let bx = crate::types::higher_order::lagrange_1d(self.order, xi);
            let by = crate::types::higher_order::lagrange_1d(self.order, eta);
            let bz = crate::types::higher_order::lagrange_1d(self.order, zeta);
            let mut result = Vec::with_capacity(self.dofs_per_cell);
            for k in 0..=self.order {
                for j in 0..=self.order {
                    for i in 0..=self.order {
                        result.push(bx[i] * by[j] * bz[k]);
                    }
                }
            }
            result
        } else {
            lagrange_simplex_basis(self.order, &self.reference_points, [xi, eta, zeta], 3)
        }
    }
}

fn lagrange_simplex_basis(
    order: usize,
    reference_points: &[[f64; 3]],
    xi: [f64; 3],
    dimension: usize,
) -> Vec<f64> {
    let exponents = simplex_monomial_exponents(order, dimension);
    if exponents.len() != reference_points.len() {
        return nearest_reference_basis(reference_points, xi);
    }

    let n = reference_points.len();
    let mut matrix = vec![vec![0.0; n + 1]; n];
    for row in 0..n {
        for (col, exp) in exponents.iter().enumerate() {
            matrix[row][col] = monomial(reference_points[row], exp);
        }
    }

    let values: Vec<f64> = exponents.iter().map(|exp| monomial(xi, exp)).collect();
    let mut basis = vec![0.0; n];
    for target in 0..n {
        for row in 0..n {
            matrix[row][n] = if row == target { 1.0 } else { 0.0 };
        }
        let Some(coeffs) = solve_augmented(matrix.clone()) else {
            return nearest_reference_basis(reference_points, xi);
        };
        basis[target] = coeffs
            .iter()
            .zip(values.iter())
            .map(|(coeff, value)| coeff * value)
            .sum();
    }
    basis
}

fn simplex_monomial_exponents(order: usize, dimension: usize) -> Vec<[usize; 3]> {
    let mut exponents = Vec::new();
    for z in 0..=order {
        for y in 0..=order - z {
            for x in 0..=order - y - z {
                if dimension == 2 && z != 0 {
                    continue;
                }
                exponents.push([x, y, z]);
            }
        }
    }
    exponents
}

fn monomial(point: [f64; 3], exp: &[usize; 3]) -> f64 {
    point[0].powi(exp[0] as i32) * point[1].powi(exp[1] as i32) * point[2].powi(exp[2] as i32)
}

fn solve_augmented(mut matrix: Vec<Vec<f64>>) -> Option<Vec<f64>> {
    let n = matrix.len();
    for col in 0..n {
        let pivot =
            (col..n).max_by(|&a, &b| matrix[a][col].abs().total_cmp(&matrix[b][col].abs()))?;
        if matrix[pivot][col].abs() <= 1e-14 {
            return None;
        }
        matrix.swap(col, pivot);
        let scale = matrix[col][col];
        for j in col..=n {
            matrix[col][j] /= scale;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = matrix[row][col];
            for j in col..=n {
                matrix[row][j] -= factor * matrix[col][j];
            }
        }
    }
    Some(matrix.into_iter().map(|row| row[n]).collect())
}

fn nearest_reference_basis(reference_points: &[[f64; 3]], xi: [f64; 3]) -> Vec<f64> {
    let mut basis = vec![0.0; reference_points.len()];
    if let Some((idx, _)) = reference_points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| squared_distance(**a, xi).total_cmp(&squared_distance(**b, xi)))
    {
        basis[idx] = 1.0;
    }
    basis
}

fn squared_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// A single DG cell with its own DOF coordinates and field data.
#[derive(Debug, Clone)]
pub struct DGCell {
    /// Physical coordinates of DOF nodes: dofs_per_cell × 3.
    pub coordinates: Vec<[f64; 3]>,
    /// Scalar/vector fields at DOF nodes. Key = field name, Value = flat f64 data.
    pub fields: HashMap<String, Vec<f64>>,
}

/// A grid of discontinuous Galerkin cells.
///
/// Each cell has its own DOF nodes (not shared), enabling
/// discontinuous field representations across cell boundaries.
#[derive(Debug, Clone)]
pub struct CellGrid {
    /// Cell specification (shared by all cells of the same type).
    pub spec: CellSpec,
    /// Individual cells with their DOF data.
    pub cells: Vec<DGCell>,
    /// Global field data (not per-DOF).
    pub field_data: FieldData,
}

impl CellGrid {
    /// Create an empty CellGrid with the given cell spec.
    pub fn new(spec: CellSpec) -> Self {
        assert!(spec.validate(), "invalid CellGrid cell specification");
        Self {
            spec,
            cells: Vec::new(),
            field_data: FieldData::new(),
        }
    }

    /// Add a cell with its DOF coordinates.
    pub fn add_cell(&mut self, coordinates: Vec<[f64; 3]>) {
        assert_eq!(
            coordinates.len(),
            self.spec.dofs_per_cell,
            "expected {} DOF coordinates, got {}",
            self.spec.dofs_per_cell,
            coordinates.len()
        );
        self.cells.push(DGCell {
            coordinates,
            fields: HashMap::new(),
        });
    }

    /// Add a cell with coordinates and a scalar field.
    pub fn add_cell_with_field(
        &mut self,
        coordinates: Vec<[f64; 3]>,
        field_name: &str,
        field_values: Vec<f64>,
    ) {
        assert_eq!(coordinates.len(), self.spec.dofs_per_cell);
        assert_eq!(field_values.len(), self.spec.dofs_per_cell);
        let mut fields = HashMap::new();
        fields.insert(field_name.to_string(), field_values);
        self.cells.push(DGCell {
            coordinates,
            fields,
        });
    }

    /// Set a field on all cells. `values` must have length `num_cells * dofs_per_cell`.
    pub fn set_field(&mut self, name: &str, values: &[f64]) {
        let dpn = self.spec.dofs_per_cell;
        assert_eq!(values.len(), self.cells.len() * dpn);
        for (i, cell) in self.cells.iter_mut().enumerate() {
            cell.fields
                .insert(name.to_string(), values[i * dpn..(i + 1) * dpn].to_vec());
        }
    }

    /// Evaluate a field at a parametric point within a cell.
    pub fn evaluate_field(&self, cell_idx: usize, field_name: &str, xi: [f64; 3]) -> Option<f64> {
        let cell = self.cells.get(cell_idx)?;
        let values = cell.fields.get(field_name)?;
        if values.len() != self.spec.dofs_per_cell {
            return None;
        }
        let basis = self.spec.evaluate_basis(xi);
        let mut result = 0.0;
        for (b, v) in basis.iter().zip(values.iter()) {
            result += b * v;
        }
        Some(result)
    }

    /// Evaluate physical coordinates at a parametric point within a cell.
    pub fn evaluate_position(&self, cell_idx: usize, xi: [f64; 3]) -> Option<[f64; 3]> {
        let cell = self.cells.get(cell_idx)?;
        let basis = self.spec.evaluate_basis(xi);
        let mut pos = [0.0; 3];
        for (b, coord) in basis.iter().zip(cell.coordinates.iter()) {
            pos[0] += b * coord[0];
            pos[1] += b * coord[1];
            pos[2] += b * coord[2];
        }
        Some(pos)
    }

    /// Number of cells.
    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    /// Total number of DOF nodes across all cells.
    pub fn total_dofs(&self) -> usize {
        self.cells.len() * self.spec.dofs_per_cell
    }

    /// Polynomial order.
    pub fn order(&self) -> usize {
        self.spec.order
    }

    /// Bounding box of all DOF coordinates.
    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for cell in &self.cells {
            for p in &cell.coordinates {
                bb.expand(*p);
            }
        }
        bb
    }

    /// Convert to linear mesh by sampling at reference vertices only.
    /// Returns (points, cell_connectivity) suitable for PolyData/UnstructuredGrid.
    pub fn to_linear_mesh(&self) -> (Vec<[f64; 3]>, Vec<Vec<usize>>) {
        let mut points = Vec::new();
        let mut cells = Vec::new();

        let corners = self.corner_indices();

        for cell in &self.cells {
            let base = points.len();
            let mut conn = Vec::new();
            for &ci in &corners {
                if ci < cell.coordinates.len() {
                    points.push(cell.coordinates[ci]);
                    conn.push(base + conn.len());
                }
            }
            cells.push(conn);
        }

        (points, cells)
    }

    /// Tessellate each cell into linear sub-cells for visualization.
    /// Subdivides each parametric cell into `subdivisions^dim` sub-cells.
    pub fn tessellate(&self, subdivisions: usize) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        assert!(subdivisions > 0, "subdivisions must be greater than zero");
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        let n = subdivisions;

        for cell_idx in 0..self.cells.len() {
            match self.spec.dimension {
                2 => {
                    let base = points.len();
                    if self.spec.cell_type == CellType::LagrangeTriangle {
                        let mut row_start = Vec::with_capacity(n + 1);
                        for j in 0..=n {
                            row_start.push(points.len());
                            for i in 0..=n - j {
                                let xi = [i as f64 / n as f64, j as f64 / n as f64, 0.0];
                                if let Some(pos) = self.evaluate_position(cell_idx, xi) {
                                    points.push(pos);
                                }
                            }
                        }
                        for j in 0..n {
                            for i in 0..n - j {
                                let v00 = row_start[j] + i;
                                let v10 = v00 + 1;
                                let v01 = row_start[j + 1] + i;
                                triangles.push([v00, v10, v01]);
                                if i + j + 1 < n {
                                    let v11 = v01 + 1;
                                    triangles.push([v10, v11, v01]);
                                }
                            }
                        }
                    } else {
                        // Tensor-product quads tessellate as a parametric grid.
                        for j in 0..=n {
                            for i in 0..=n {
                                let xi = [i as f64 / n as f64, j as f64 / n as f64, 0.0];
                                if let Some(pos) = self.evaluate_position(cell_idx, xi) {
                                    points.push(pos);
                                }
                            }
                        }
                        for j in 0..n {
                            for i in 0..n {
                                let v00 = base + j * (n + 1) + i;
                                let v10 = v00 + 1;
                                let v01 = v00 + (n + 1);
                                let v11 = v01 + 1;
                                triangles.push([v00, v10, v11]);
                                triangles.push([v00, v11, v01]);
                            }
                        }
                    }
                }
                _ => {
                    // For 1D/3D, just add corner points
                    if let Some(pos) = self.evaluate_position(cell_idx, [0.0, 0.0, 0.0]) {
                        points.push(pos);
                    }
                }
            }
        }

        (points, triangles)
    }

    fn corner_indices(&self) -> Vec<usize> {
        let targets: &[[f64; 3]] = match self.spec.cell_type {
            CellType::LagrangeCurve => &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            CellType::LagrangeTriangle => &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            CellType::LagrangeQuadrilateral => &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            CellType::LagrangeTetrahedron => &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            CellType::LagrangeHexahedron => &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            _ => &[[0.0, 0.0, 0.0]],
        };
        targets
            .iter()
            .filter_map(|target| {
                self.spec
                    .reference_points
                    .iter()
                    .position(|point| squared_distance(*point, *target) <= 1e-24)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lagrange_triangle_p1() {
        let spec = CellSpec::lagrange_triangle(1);
        assert_eq!(spec.dofs_per_cell, 3);
        assert_eq!(spec.order, 1);
        assert_eq!(spec.dimension, 2);
    }

    #[test]
    #[should_panic(expected = "Lagrange cell order must be greater than zero")]
    fn lagrange_order_zero_rejected() {
        CellSpec::lagrange_quad(0);
    }

    #[test]
    #[should_panic(expected = "invalid CellGrid cell specification")]
    fn cell_grid_rejects_unsupported_custom_spec() {
        CellGrid::new(CellSpec {
            cell_type: CellType::BezierCurve,
            order: 1,
            dofs_per_cell: 2,
            reference_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            dimension: 1,
        });
    }

    #[test]
    fn lagrange_triangle_p2() {
        let spec = CellSpec::lagrange_triangle(2);
        assert_eq!(spec.dofs_per_cell, 6); // (2+1)(2+2)/2 = 6
    }

    #[test]
    fn lagrange_hex_p1() {
        let spec = CellSpec::lagrange_hex(1);
        assert_eq!(spec.dofs_per_cell, 8);
    }

    #[test]
    fn lagrange_hex_p2() {
        let spec = CellSpec::lagrange_hex(2);
        assert_eq!(spec.dofs_per_cell, 27); // 3^3
    }

    #[test]
    fn cell_grid_add_and_evaluate() {
        let spec = CellSpec::lagrange_quad(1);
        let mut grid = CellGrid::new(spec);

        // Unit square with linear field f(x,y) = x + y
        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let field = vec![0.0, 1.0, 1.0, 2.0]; // f = x + y at corners
        grid.add_cell_with_field(coords, "f", field);

        assert_eq!(grid.num_cells(), 1);
        assert_eq!(grid.total_dofs(), 4);

        // Evaluate at center: f(0.5, 0.5) should be ~1.0
        let val = grid.evaluate_field(0, "f", [0.5, 0.5, 0.0]).unwrap();
        assert!(
            (val - 1.0).abs() < 1e-10,
            "f(0.5,0.5) = {val}, expected 1.0"
        );

        // Evaluate position at center
        let pos = grid.evaluate_position(0, [0.5, 0.5, 0.0]).unwrap();
        assert!((pos[0] - 0.5).abs() < 1e-10);
        assert!((pos[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cell_grid_set_field() {
        let spec = CellSpec::lagrange_curve(2);
        let mut grid = CellGrid::new(spec);
        grid.add_cell(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        grid.add_cell(vec![[1.0, 0.0, 0.0], [1.5, 0.0, 0.0], [2.0, 0.0, 0.0]]);

        grid.set_field("u", &[0.0, 0.25, 1.0, 1.0, 2.25, 4.0]); // u = x^2

        let v0 = grid.evaluate_field(0, "u", [0.5, 0.0, 0.0]).unwrap();
        assert!((v0 - 0.25).abs() < 0.1, "u(0.5) ≈ 0.25, got {v0}");
    }

    #[test]
    fn cell_grid_bounds() {
        let spec = CellSpec::lagrange_triangle(1);
        let mut grid = CellGrid::new(spec);
        grid.add_cell(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);

        let bb = grid.bounds();
        assert!((bb.size()[0] - 1.0).abs() < 1e-10);
        assert!((bb.size()[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tessellate_quad() {
        let spec = CellSpec::lagrange_quad(1);
        let mut grid = CellGrid::new(spec);
        grid.add_cell(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);

        let (pts, tris) = grid.tessellate(4);
        assert_eq!(pts.len(), 25); // (4+1)^2
        assert_eq!(tris.len(), 32); // 4*4*2
    }

    #[test]
    #[should_panic(expected = "subdivisions must be greater than zero")]
    fn tessellate_zero_subdivisions_rejected() {
        let spec = CellSpec::lagrange_quad(1);
        let grid = CellGrid::new(spec);
        grid.tessellate(0);
    }

    #[test]
    fn to_linear_mesh() {
        let spec = CellSpec::lagrange_quad(2);
        let mut grid = CellGrid::new(spec);
        // 9-node biquadratic quad
        let mut coords = Vec::new();
        for j in 0..=2 {
            for i in 0..=2 {
                coords.push([i as f64, j as f64, 0.0]);
            }
        }
        grid.add_cell(coords);

        let (_pts, cells) = grid.to_linear_mesh();
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].len(), 4); // extracted 4 corners from 9-node quad
    }

    #[test]
    fn to_linear_mesh_uses_vtk_quad_corner_order() {
        let spec = CellSpec::lagrange_quad(2);
        let mut grid = CellGrid::new(spec);
        let mut coords = Vec::new();
        for j in 0..=2 {
            for i in 0..=2 {
                coords.push([i as f64, j as f64, 0.0]);
            }
        }
        grid.add_cell(coords);

        let (pts, cells) = grid.to_linear_mesh();

        assert_eq!(cells[0], vec![0, 1, 2, 3]);
        assert_eq!(
            pts,
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ]
        );
    }
}
