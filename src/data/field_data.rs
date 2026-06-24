use crate::data::AnyDataArray;

/// A named, heterogeneous collection of data arrays.
///
/// Analogous to VTK's `vtkFieldData`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldData {
    arrays: Vec<AnyDataArray>,
}

impl FieldData {
    pub fn new() -> Self {
        Self { arrays: Vec::new() }
    }

    pub fn add_array(&mut self, array: AnyDataArray) {
        // Replace existing array with the same name.
        if let Some(pos) = self.arrays.iter().position(|a| a.name() == array.name()) {
            self.arrays[pos] = array;
        } else {
            self.arrays.push(array);
        }
    }

    pub fn get_array(&self, name: &str) -> Option<&AnyDataArray> {
        self.arrays.iter().find(|a| a.name() == name)
    }

    pub fn get_array_mut(&mut self, name: &str) -> Option<&mut AnyDataArray> {
        self.arrays.iter_mut().find(|a| a.name() == name)
    }

    pub fn get_array_by_index(&self, idx: usize) -> Option<&AnyDataArray> {
        self.arrays.get(idx)
    }

    pub fn num_arrays(&self) -> usize {
        self.arrays.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnyDataArray> {
        self.arrays.iter()
    }

    pub fn remove_array(&mut self, name: &str) -> Option<AnyDataArray> {
        if let Some(pos) = self.arrays.iter().position(|a| a.name() == name) {
            Some(self.arrays.remove(pos))
        } else {
            None
        }
    }

    /// Check if an array with the given name exists.
    pub fn has_array(&self, name: &str) -> bool {
        self.arrays.iter().any(|a| a.name() == name)
    }

    /// Get all array names.
    pub fn names(&self) -> Vec<&str> {
        self.arrays.iter().map(|a| a.name()).collect()
    }

    /// Check if the field data is empty.
    pub fn is_empty(&self) -> bool {
        self.arrays.is_empty()
    }

    /// Clear all arrays.
    pub fn clear(&mut self) {
        self.arrays.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn add_and_get() {
        let mut fd = FieldData::new();
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "temp",
            vec![1.0, 2.0],
            1,
        )));
        assert_eq!(fd.num_arrays(), 1);
        assert!(fd.has_array("temp"));
        assert!(!fd.has_array("other"));
    }

    #[test]
    fn replace_same_name() {
        let mut fd = FieldData::new();
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0],
            1,
        )));
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![2.0, 3.0],
            1,
        )));
        assert_eq!(fd.num_arrays(), 1);
        assert_eq!(fd.get_array("x").unwrap().num_tuples(), 2);
    }

    #[test]
    fn remove() {
        let mut fd = FieldData::new();
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "a",
            vec![1.0],
            1,
        )));
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "b",
            vec![2.0],
            1,
        )));
        fd.remove_array("a");
        assert_eq!(fd.num_arrays(), 1);
        assert!(!fd.has_array("a"));
    }

    #[test]
    fn names() {
        let mut fd = FieldData::new();
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0],
            1,
        )));
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "y",
            vec![2.0],
            1,
        )));
        assert_eq!(fd.names(), vec!["x", "y"]);
    }

    #[test]
    fn clear() {
        let mut fd = FieldData::new();
        fd.add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0],
            1,
        )));
        assert!(!fd.is_empty());
        fd.clear();
        assert!(fd.is_empty());
    }
}
