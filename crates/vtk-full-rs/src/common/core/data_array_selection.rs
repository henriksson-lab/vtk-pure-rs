use super::{object::Object, vtk_type::VtkMTimeType};

/// VTK: `vtkDataArraySelection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataArraySelection {
    object: Object,
    arrays: Vec<(String, bool)>,
    unknown_array_setting: i32,
}

impl DataArraySelection {
    /// VTK: `vtkDataArraySelection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataArraySelection"),
            arrays: Vec::new(),
            unknown_array_setting: 0,
        }
    }

    /// VTK: `vtkDataArraySelection::EnableArray`.
    pub fn enable_array(&mut self, name: Option<&str>) {
        self.set_array_setting(name, 1);
    }

    /// VTK: `vtkDataArraySelection::DisableArray`.
    pub fn disable_array(&mut self, name: Option<&str>) {
        self.set_array_setting(name, 0);
    }

    /// VTK: `vtkDataArraySelection::ArrayIsEnabled`.
    pub fn array_is_enabled(&self, name: Option<&str>) -> i32 {
        self.find(name).map_or(self.unknown_array_setting, |index| {
            self.arrays[index].1 as i32
        })
    }

    /// VTK: `vtkDataArraySelection::ArrayExists`.
    pub fn array_exists(&self, name: Option<&str>) -> i32 {
        self.find(name).is_some() as i32
    }

    /// VTK: `vtkDataArraySelection::EnableAllArrays`.
    pub fn enable_all_arrays(&mut self) {
        let mut modified = false;
        for (_, enabled) in &mut self.arrays {
            if !*enabled {
                *enabled = true;
                modified = true;
            }
        }
        if modified {
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::DisableAllArrays`.
    pub fn disable_all_arrays(&mut self) {
        let mut modified = false;
        for (_, enabled) in &mut self.arrays {
            if *enabled {
                *enabled = false;
                modified = true;
            }
        }
        if modified {
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::GetNumberOfArrays`.
    pub fn get_number_of_arrays(&self) -> i32 {
        self.arrays.len() as i32
    }

    /// VTK: `vtkDataArraySelection::GetNumberOfArraysEnabled`.
    pub fn get_number_of_arrays_enabled(&self) -> i32 {
        self.arrays
            .iter()
            .map(|(_, enabled)| i32::from(*enabled))
            .sum()
    }

    /// VTK: `vtkDataArraySelection::GetArrayName`.
    pub fn get_array_name(&self, index: i32) -> Option<&str> {
        if index < 0 {
            return None;
        }
        self.arrays
            .get(index as usize)
            .map(|(name, _)| name.as_str())
    }

    /// VTK: `vtkDataArraySelection::GetArrayIndex`.
    pub fn get_array_index(&self, name: Option<&str>) -> i32 {
        self.find(name).map_or(-1, |index| index as i32)
    }

    /// VTK: `vtkDataArraySelection::GetEnabledArrayIndex`.
    pub fn get_enabled_array_index(&self, name: &str) -> i32 {
        let mut index = 0;
        for (array_name, enabled) in &self.arrays {
            if array_name == name {
                return index;
            }
            if *enabled {
                index += 1;
            }
        }
        -1
    }

    /// VTK: `vtkDataArraySelection::GetArraySetting(int)`.
    pub fn get_array_setting(&self, index: i32) -> i32 {
        if index >= 0 && index < self.get_number_of_arrays() {
            i32::from(self.arrays[index as usize].1)
        } else {
            0
        }
    }

    /// VTK: `vtkDataArraySelection::GetArraySetting(const char*)`.
    pub fn get_array_setting_by_name(&self, name: Option<&str>) -> i32 {
        self.array_is_enabled(name)
    }

    /// VTK: `vtkDataArraySelection::SetArraySetting`.
    pub fn set_array_setting(&mut self, name: Option<&str>, setting: i32) {
        let status = setting > 0;
        if let Some(index) = self.find(name) {
            if self.arrays[index].1 != status {
                self.arrays[index].1 = status;
                self.modified();
            }
        } else if let Some(name) = name {
            self.arrays.push((name.to_string(), status));
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::RemoveAllArrays`.
    pub fn remove_all_arrays(&mut self) {
        if !self.arrays.is_empty() {
            self.arrays.clear();
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::AddArray`.
    pub fn add_array(&mut self, name: &str, state: bool) -> i32 {
        if self.array_exists(Some(name)) != 0 {
            return 0;
        }
        self.arrays.push((name.to_string(), state));
        1
    }

    /// VTK: `vtkDataArraySelection::RemoveArrayByIndex`.
    pub fn remove_array_by_index(&mut self, index: i32) {
        if index >= 0 && index < self.get_number_of_arrays() {
            self.arrays.remove(index as usize);
        }
    }

    /// VTK: `vtkDataArraySelection::RemoveArrayByName`.
    pub fn remove_array_by_name(&mut self, name: Option<&str>) {
        if let Some(index) = self.find(name) {
            self.arrays.remove(index);
        }
    }

    /// VTK: `vtkDataArraySelection::SetArrays`.
    pub fn set_arrays(&mut self, names: &[&str]) {
        self.set_arrays_with_default(names, 1);
    }

    /// VTK: `vtkDataArraySelection::SetArraysWithDefault`.
    pub fn set_arrays_with_default(&mut self, names: &[&str], default_status: i32) {
        let mut arrays = Vec::with_capacity(names.len());
        for name in names {
            let setting = self
                .find(Some(name))
                .map_or(default_status != 0, |index| self.arrays[index].1);
            arrays.push(((*name).to_string(), setting));
        }
        self.arrays = arrays;
    }

    /// VTK: `vtkDataArraySelection::CopySelections`.
    pub fn copy_selections(&mut self, selections: &Self) {
        if std::ptr::eq(self, selections) {
            return;
        }

        let mut need_update = self.get_number_of_arrays() != selections.get_number_of_arrays();
        if !need_update {
            for (array_name, _) in &self.arrays {
                if selections.array_exists(Some(array_name)) == 0 {
                    need_update = true;
                    break;
                }
                if selections.array_is_enabled(Some(array_name))
                    != self.array_is_enabled(Some(array_name))
                {
                    need_update = true;
                    break;
                }
            }
        }

        if need_update {
            self.arrays.clone_from(&selections.arrays);
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::Union`.
    pub fn union(&mut self, other: &Self) {
        self.union_with_skip_modified(other, false);
    }

    /// VTK: `vtkDataArraySelection::Union(vtkDataArraySelection*, bool)`.
    pub fn union_with_skip_modified(&mut self, other: &Self, skip_modified: bool) {
        let mut modified = false;
        for (name, enabled) in &other.arrays {
            if self.find(Some(name)).is_none() {
                self.arrays.push((name.clone(), *enabled));
                modified = true;
            }
        }

        if modified && !skip_modified {
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::SetUnknownArraySetting`.
    pub fn set_unknown_array_setting(&mut self, value: i32) {
        if self.unknown_array_setting != value {
            self.unknown_array_setting = value;
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::GetUnknownArraySetting`.
    pub fn get_unknown_array_setting(&self) -> i32 {
        self.unknown_array_setting
    }

    /// VTK: `vtkDataArraySelection::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        if !self.is_equal(other) {
            self.unknown_array_setting = other.unknown_array_setting;
            self.arrays.clone_from(&other.arrays);
            self.modified();
        }
    }

    /// VTK: `vtkDataArraySelection::IsEqual`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.unknown_array_setting == other.unknown_array_setting && self.arrays == other.arrays
    }

    /// VTK: `vtkDataArraySelection::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = format!(
            "UnknownArraySetting: {}\nNumber of Arrays: {}\n",
            self.unknown_array_setting,
            self.get_number_of_arrays()
        );
        for (name, enabled) in &self.arrays {
            let setting = if *enabled { "enabled" } else { "disabled" };
            out.push_str(&format!(
                "Array: {} is: {} ({})\n",
                name,
                setting,
                self.array_is_enabled(Some(name))
            ));
        }
        out
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    fn find(&self, name: Option<&str>) -> Option<usize> {
        name.and_then(|name| {
            self.arrays
                .iter()
                .position(|(array_name, _)| array_name == name)
        })
    }
}

impl Default for DataArraySelection {
    fn default() -> Self {
        Self::new()
    }
}
