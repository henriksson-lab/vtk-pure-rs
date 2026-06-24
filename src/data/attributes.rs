use crate::data::{AnyDataArray, FieldData};

/// Field data with "active" attribute designations for scalars, vectors, normals, etc.
///
/// Analogous to VTK's `vtkDataSetAttributes`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DataSetAttributes {
    field_data: FieldData,
    active_scalars: Option<String>,
    active_vectors: Option<String>,
    active_normals: Option<String>,
    active_tcoords: Option<String>,
    active_tensors: Option<String>,
    active_global_ids: Option<String>,
    active_pedigree_ids: Option<String>,
    active_edge_flags: Option<String>,
    active_tangents: Option<String>,
    active_rational_weights: Option<String>,
    active_higher_order_degrees: Option<String>,
    active_process_ids: Option<String>,
}

impl DataSetAttributes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }

    pub fn add_array(&mut self, array: AnyDataArray) {
        self.field_data.add_array(array);
    }

    pub fn num_arrays(&self) -> usize {
        self.field_data.num_arrays()
    }

    pub fn get_array(&self, name: &str) -> Option<&AnyDataArray> {
        self.field_data.get_array(name)
    }

    pub fn get_array_by_index(&self, idx: usize) -> Option<&AnyDataArray> {
        self.field_data.get_array_by_index(idx)
    }

    // Active attribute setters/getters

    pub fn set_active_scalars(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::NoLimit, 0);
        Self::set_active_attribute(&mut self.active_scalars, name, check)
    }

    pub fn set_active_vectors(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 3);
        Self::set_active_attribute(&mut self.active_vectors, name, check)
    }

    pub fn set_active_normals(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 3);
        Self::set_active_attribute(&mut self.active_normals, name, check)
    }

    pub fn set_active_tcoords(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Max, 3);
        Self::set_active_attribute(&mut self.active_tcoords, name, check)
    }

    pub fn set_active_tensors(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Tensor, 9);
        Self::set_active_attribute(&mut self.active_tensors, name, check)
    }

    pub fn set_active_global_ids(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 1);
        Self::set_active_attribute(&mut self.active_global_ids, name, check)
    }

    pub fn set_active_pedigree_ids(&mut self, name: &str) -> bool {
        let check = if self.has_array(name) {
            AttributeCheck::Valid
        } else {
            AttributeCheck::Missing
        };
        Self::set_active_attribute(&mut self.active_pedigree_ids, name, check)
    }

    pub fn set_active_edge_flags(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 1);
        Self::set_active_attribute(&mut self.active_edge_flags, name, check)
    }

    pub fn set_active_tangents(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 3);
        Self::set_active_attribute(&mut self.active_tangents, name, check)
    }

    pub fn set_active_rational_weights(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 1);
        Self::set_active_attribute(&mut self.active_rational_weights, name, check)
    }

    pub fn set_active_higher_order_degrees(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 3);
        Self::set_active_attribute(&mut self.active_higher_order_degrees, name, check)
    }

    pub fn set_active_process_ids(&mut self, name: &str) -> bool {
        let check = self.check_attribute_components(name, AttributeLimit::Exact, 1);
        Self::set_active_attribute(&mut self.active_process_ids, name, check)
    }

    pub fn scalars(&self) -> Option<&AnyDataArray> {
        self.active_scalars
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn vectors(&self) -> Option<&AnyDataArray> {
        self.active_vectors
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn normals(&self) -> Option<&AnyDataArray> {
        self.active_normals
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn tcoords(&self) -> Option<&AnyDataArray> {
        self.active_tcoords
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn tensors(&self) -> Option<&AnyDataArray> {
        self.active_tensors
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn global_ids(&self) -> Option<&AnyDataArray> {
        self.active_global_ids
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn pedigree_ids(&self) -> Option<&AnyDataArray> {
        self.active_pedigree_ids
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn edge_flags(&self) -> Option<&AnyDataArray> {
        self.active_edge_flags
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn tangents(&self) -> Option<&AnyDataArray> {
        self.active_tangents
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn rational_weights(&self) -> Option<&AnyDataArray> {
        self.active_rational_weights
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn higher_order_degrees(&self) -> Option<&AnyDataArray> {
        self.active_higher_order_degrees
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    pub fn process_ids(&self) -> Option<&AnyDataArray> {
        self.active_process_ids
            .as_deref()
            .and_then(|name| self.field_data.get_array(name))
    }

    /// Check if an array with the given name exists.
    pub fn has_array(&self, name: &str) -> bool {
        self.field_data.has_array(name)
    }

    /// Get all array names.
    pub fn array_names(&self) -> Vec<&str> {
        self.field_data.names()
    }

    /// Remove an array by name. Adjusts active attribute indices.
    pub fn remove_array(&mut self, name: &str) -> Option<AnyDataArray> {
        let result = self.field_data.remove_array(name);
        if result.is_some() {
            self.clear_active_name(name);
        }
        result
    }

    /// Remove all arrays and clear active attributes.
    pub fn clear(&mut self) {
        self.field_data.clear();
        self.active_scalars = None;
        self.active_vectors = None;
        self.active_normals = None;
        self.active_tcoords = None;
        self.active_tensors = None;
        self.active_global_ids = None;
        self.active_pedigree_ids = None;
        self.active_edge_flags = None;
        self.active_tangents = None;
        self.active_rational_weights = None;
        self.active_higher_order_degrees = None;
        self.active_process_ids = None;
    }

    /// Check if any active attributes are set.
    pub fn has_active_attributes(&self) -> bool {
        self.scalars().is_some()
            || self.vectors().is_some()
            || self.normals().is_some()
            || self.tcoords().is_some()
            || self.tensors().is_some()
            || self.global_ids().is_some()
            || self.pedigree_ids().is_some()
            || self.edge_flags().is_some()
            || self.tangents().is_some()
            || self.rational_weights().is_some()
            || self.higher_order_degrees().is_some()
            || self.process_ids().is_some()
    }

    /// Iterate over all arrays.
    pub fn iter(&self) -> impl Iterator<Item = &AnyDataArray> {
        self.field_data.iter()
    }

    fn clear_active_name(&mut self, name: &str) {
        fn clear_if_matches(active: &mut Option<String>, name: &str) {
            if active.as_deref() == Some(name) {
                *active = None;
            }
        }
        clear_if_matches(&mut self.active_scalars, name);
        clear_if_matches(&mut self.active_vectors, name);
        clear_if_matches(&mut self.active_normals, name);
        clear_if_matches(&mut self.active_tcoords, name);
        clear_if_matches(&mut self.active_tensors, name);
        clear_if_matches(&mut self.active_global_ids, name);
        clear_if_matches(&mut self.active_pedigree_ids, name);
        clear_if_matches(&mut self.active_edge_flags, name);
        clear_if_matches(&mut self.active_tangents, name);
        clear_if_matches(&mut self.active_rational_weights, name);
        clear_if_matches(&mut self.active_higher_order_degrees, name);
        clear_if_matches(&mut self.active_process_ids, name);
    }

    fn set_active_attribute(
        active: &mut Option<String>,
        name: &str,
        check: AttributeCheck,
    ) -> bool {
        match check {
            AttributeCheck::Valid => {
                *active = Some(name.to_string());
                true
            }
            AttributeCheck::Missing => {
                *active = None;
                false
            }
            AttributeCheck::InvalidComponents => false,
        }
    }

    fn check_attribute_components(
        &self,
        name: &str,
        limit: AttributeLimit,
        expected: usize,
    ) -> AttributeCheck {
        let Some(array) = self.field_data.get_array(name) else {
            return AttributeCheck::Missing;
        };
        let num_components = array.num_components();
        let valid = match limit {
            AttributeLimit::NoLimit => true,
            AttributeLimit::Max => num_components <= expected,
            AttributeLimit::Exact => num_components == expected,
            AttributeLimit::Tensor => num_components == expected || num_components == 6,
        };
        if valid {
            AttributeCheck::Valid
        } else {
            AttributeCheck::InvalidComponents
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AttributeLimit {
    Max,
    Exact,
    NoLimit,
    Tensor,
}

#[derive(Debug, Clone, Copy)]
enum AttributeCheck {
    Valid,
    Missing,
    InvalidComponents,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn add_and_get() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "temp",
            vec![1.0, 2.0, 3.0],
            1,
        )));
        assert_eq!(attrs.num_arrays(), 1);
        assert!(attrs.has_array("temp"));
    }

    #[test]
    fn active_scalars() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("temp", vec![1.0], 1)));
        attrs.set_active_scalars("temp");
        assert!(attrs.scalars().is_some());
        assert_eq!(attrs.scalars().unwrap().name(), "temp");
    }

    #[test]
    fn active_normals() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "N",
            vec![0.0, 0.0, 1.0],
            3,
        )));
        attrs.set_active_normals("N");
        assert!(attrs.normals().is_some());
    }

    #[test]
    fn active_attribute_component_limits_match_vtk() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "bad_v",
            vec![1.0, 2.0],
            2,
        )));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "v",
            vec![1.0, 2.0, 3.0],
            3,
        )));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "tc",
            vec![1.0, 2.0],
            2,
        )));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "tensor6",
            vec![0.0; 6],
            6,
        )));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("ids", vec![7.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("edge", vec![1.0], 1)));

        assert!(!attrs.set_active_vectors("bad_v"));
        assert!(attrs.vectors().is_none());
        assert!(attrs.set_active_vectors("v"));
        assert!(attrs.set_active_tcoords("tc"));
        assert!(attrs.set_active_tensors("tensor6"));
        assert!(attrs.set_active_global_ids("ids"));
        assert!(attrs.global_ids().is_some());
        assert!(attrs.set_active_edge_flags("edge"));
        assert!(attrs.edge_flags().is_some());
    }

    #[test]
    fn missing_active_attribute_name_clears_like_vtk() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "v",
            vec![1.0, 2.0, 3.0],
            3,
        )));
        assert!(attrs.set_active_vectors("v"));

        assert!(!attrs.set_active_vectors("missing"));

        assert!(attrs.vectors().is_none());
        assert!(!attrs.has_active_attributes());
    }

    #[test]
    fn invalid_active_attribute_components_preserve_existing_like_vtk() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "v",
            vec![1.0, 2.0, 3.0],
            3,
        )));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            "bad_v",
            vec![1.0, 2.0],
            2,
        )));
        assert!(attrs.set_active_vectors("v"));

        assert!(!attrs.set_active_vectors("bad_v"));

        assert_eq!(attrs.vectors().unwrap().name(), "v");
    }

    #[test]
    fn remove_adjusts_active() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![2.0], 1)));
        attrs.set_active_scalars("b");
        attrs.remove_array("a");
        assert!(attrs.scalars().is_some());
        assert_eq!(attrs.scalars().unwrap().name(), "b");
    }

    #[test]
    fn active_names_survive_external_field_data_mutation() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![2.0], 1)));
        attrs.set_active_scalars("b");

        attrs.field_data_mut().remove_array("a");

        assert_eq!(attrs.scalars().unwrap().name(), "b");
    }

    #[test]
    fn active_names_do_not_retarget_after_external_remove() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![2.0], 1)));
        attrs.set_active_scalars("a");

        attrs.field_data_mut().remove_array("a");

        assert!(attrs.scalars().is_none());
        assert!(!attrs.has_active_attributes());
    }

    #[test]
    fn remove_active_clears() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0], 1)));
        attrs.set_active_scalars("x");
        attrs.remove_array("x");
        assert!(attrs.scalars().is_none());
    }

    #[test]
    fn array_names() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("b", vec![2.0], 1)));
        assert_eq!(attrs.array_names(), vec!["a", "b"]);
    }

    #[test]
    fn has_active() {
        let mut attrs = DataSetAttributes::new();
        assert!(!attrs.has_active_attributes());
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("s", vec![1.0], 1)));
        attrs.set_active_scalars("s");
        assert!(attrs.has_active_attributes());
    }

    #[test]
    fn iterate() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0], 1)));
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("y", vec![2.0], 1)));
        let names: Vec<&str> = attrs.iter().map(|a| a.name()).collect();
        assert_eq!(names, vec!["x", "y"]);
    }

    #[test]
    fn clear() {
        let mut attrs = DataSetAttributes::new();
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        attrs.set_active_scalars("a");
        assert_eq!(attrs.num_arrays(), 1);
        assert!(attrs.has_active_attributes());

        attrs.clear();
        assert_eq!(attrs.num_arrays(), 0);
        assert!(!attrs.has_active_attributes());
        assert!(attrs.scalars().is_none());
    }
}
