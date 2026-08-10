use crate::common::core::{AnyArray, VtkDataType};
use crate::common::data_model::{
    DataSetAttribute, DataSetAttributeCopyOperation, DataSetAttributes, DataSetAttributesError,
    FieldData, FieldDataArray,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldListMode {
    None,
    Intersection,
    Union,
}

/// Compact metadata for one `vtkDataSetAttributesFieldList` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldInfo {
    name: String,
    number_of_components: usize,
    data_type: VtkDataType,
    attribute_types: Vec<[bool; DataSetAttribute::ALL.len()]>,
    locations: Vec<isize>,
    output_location: isize,
}

impl FieldInfo {
    fn create(array: &FieldDataArray, location: usize, attrs: &[isize]) -> Self {
        let mut attribute_types = Vec::new();
        let mut current_attrs = [false; DataSetAttribute::ALL.len()];
        for role in DataSetAttribute::ALL {
            current_attrs[role.index()] = attrs[role.index()] == location as isize;
        }
        attribute_types.push(current_attrs);

        Self {
            name: array.get_name().to_string(),
            number_of_components: array.get_number_of_components(),
            data_type: array.get_data_type(),
            attribute_types,
            locations: vec![location as isize],
            output_location: -1,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    #[cfg(test)]
    fn get_number_of_components(&self) -> usize {
        self.number_of_components
    }

    #[cfg(test)]
    fn locations(&self) -> &[isize] {
        &self.locations
    }

    #[cfg(test)]
    fn output_location(&self) -> isize {
        self.output_location
    }

    fn is_empty(&self) -> bool {
        self.data_type == VtkDataType::Void
    }

    fn is_similar(&self, other: &Self) -> bool {
        self.name == other.name
            && self.number_of_components == other.number_of_components
            && self.data_type == other.data_type
    }

    fn merge(&self, other: &Self) -> Option<Self> {
        if self.is_empty() || !self.is_similar(other) || other.locations.len() != 1 {
            return None;
        }

        let mut result = self.clone();
        result.locations.extend_from_slice(&other.locations);
        result
            .attribute_types
            .extend_from_slice(&other.attribute_types);
        Some(result)
    }

    fn extend_for_union(&mut self) {
        self.locations.push(-1);
        self.attribute_types
            .push([false; DataSetAttribute::ALL.len()]);
    }

    fn pre_extend_for_union(&mut self, count: usize) {
        let mut locations = vec![-1; count];
        locations.extend_from_slice(&self.locations);
        self.locations = locations;

        let mut attribute_types = vec![[false; DataSetAttribute::ALL.len()]; count];
        attribute_types.extend_from_slice(&self.attribute_types);
        self.attribute_types = attribute_types;
    }

    fn consistently_active_for(&self, role: DataSetAttribute) -> bool {
        !self.attribute_types.is_empty()
            && self.attribute_types.iter().all(|attrs| attrs[role.index()])
    }

    fn active_attribute_for_input(&self, input_index: usize) -> Option<DataSetAttribute> {
        let attrs = self.attribute_types.get(input_index)?;
        DataSetAttribute::ALL
            .into_iter()
            .find(|role| attrs[role.index()])
    }

    fn prototype_array(&self, tuples_to_reserve: usize) -> FieldDataArray {
        let mut array = FieldDataArray::new_with_data_type(
            &self.name,
            self.number_of_components,
            self.data_type,
        );
        array.reserve_values(tuples_to_reserve.saturating_mul(self.number_of_components));
        array
    }
}

/// VTK: `vtkDataSetAttributesFieldList`.
///
/// This compact planner keeps the VTK field-list invariants represented by the
/// new crate: same-name/same-component/same-value-kind matching, per-input
/// source locations, output locations set during allocation, and active
/// attribute roles retained only when every input marks the same field active.
#[derive(Debug, Clone, PartialEq)]
pub struct DataSetAttributesFieldList {
    fields: Vec<FieldInfo>,
    number_of_tuples: usize,
    number_of_inputs: usize,
    mode: FieldListMode,
}

impl DataSetAttributesFieldList {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            number_of_tuples: 0,
            number_of_inputs: 0,
            mode: FieldListMode::None,
        }
    }

    /// VTK: `vtkDataSetAttributesFieldList::Reset`.
    pub fn reset(&mut self) {
        self.fields.clear();
        self.number_of_tuples = 0;
        self.number_of_inputs = 0;
        self.mode = FieldListMode::None;
    }

    /// VTK: `vtkDataSetAttributesFieldList::InitializeFieldList`.
    pub fn initialize_field_list(&mut self, dsa: &DataSetAttributes) {
        self.reset();
        self.fields = Self::get_fields(dsa);
        self.number_of_tuples += dsa.field_data().get_number_of_tuples() as usize;
        self.number_of_inputs += 1;

        for field in &mut self.fields {
            field.output_location = field.locations[0];
        }
    }

    /// VTK: `vtkDataSetAttributesFieldList::IntersectFieldList`.
    pub fn intersect_field_list(&mut self, dsa: &DataSetAttributes) {
        if self.number_of_inputs == 0 {
            self.initialize_field_list(dsa);
            self.mode = FieldListMode::Intersection;
            return;
        }
        if self.mode == FieldListMode::Union {
            return;
        }

        self.mode = FieldListMode::Intersection;
        self.number_of_tuples += dsa.field_data().get_number_of_tuples() as usize;
        let current = Self::get_fields(dsa);
        let mut next_fields = Vec::new();

        for name in names_in_both(&self.fields, &current) {
            let mut accumulated_for_name: Vec<_> = self
                .fields
                .iter()
                .filter(|field| field.name == name)
                .cloned()
                .collect();
            let current_for_name: Vec<_> = current
                .iter()
                .filter(|field| field.name == name)
                .cloned()
                .collect();

            for current_field in current_for_name {
                if let Some(position) = accumulated_for_name
                    .iter()
                    .position(|field| field.is_similar(&current_field))
                {
                    let accumulated = accumulated_for_name.remove(position);
                    if let Some(merged) = accumulated.merge(&current_field) {
                        next_fields.push(merged);
                    }
                }
            }
        }

        self.fields = next_fields;
        self.number_of_inputs += 1;
    }

    /// VTK: `vtkDataSetAttributesFieldList::UnionFieldList`.
    pub fn union_field_list(&mut self, dsa: &DataSetAttributes) {
        if self.number_of_inputs == 0 {
            self.initialize_field_list(dsa);
            self.mode = FieldListMode::Union;
            return;
        }
        if self.mode == FieldListMode::Intersection {
            return;
        }

        self.mode = FieldListMode::Union;
        self.number_of_tuples += dsa.field_data().get_number_of_tuples() as usize;
        let mut current = Self::get_fields(dsa);
        let mut updated = vec![false; self.fields.len()];

        for current_field in &mut current {
            if let Some((index, accumulated)) =
                self.fields.iter_mut().enumerate().find(|(index, field)| {
                    !updated[*index]
                        && field.name == current_field.name
                        && field.is_similar(current_field)
                })
            {
                if let Some(merged) = accumulated.merge(current_field) {
                    *accumulated = merged;
                    updated[index] = true;
                    *current_field = FieldInfo {
                        name: String::new(),
                        number_of_components: 0,
                        data_type: VtkDataType::Void,
                        attribute_types: Vec::new(),
                        locations: Vec::new(),
                        output_location: -1,
                    };
                }
            }
        }

        for (index, field) in self.fields.iter_mut().enumerate() {
            if !updated[index] {
                field.extend_for_union();
            }
        }

        for mut field in current {
            if !field.is_empty() {
                field.pre_extend_for_union(self.number_of_inputs);
                self.fields.push(field);
            }
        }

        self.number_of_inputs += 1;
    }

    /// VTK: `vtkDataSetAttributesFieldList::CopyAllocate`.
    pub(crate) fn copy_allocate(
        &mut self,
        output: &mut DataSetAttributes,
        operation: DataSetAttributeCopyOperation,
        size: usize,
        _ext: usize,
    ) {
        self.fields.retain(|field| !field.is_empty());
        let size = if size > 0 {
            size
        } else {
            self.number_of_tuples
        };
        let attribute_fields = self.attribute_field_indices();

        for index in 0..self.fields.len() {
            self.fields[index].output_location = -1;
            let mut skip = false;
            let mut is_attribute = false;

            for role in DataSetAttribute::ALL {
                if attribute_fields[role.index()] == Some(index) {
                    is_attribute = true;
                    if !output.copy_attribute_role_enabled(role, operation) {
                        skip = true;
                    }
                }
            }

            if skip {
                continue;
            }

            if !is_attribute
                && !output
                    .field_data()
                    .should_copy_array(self.fields[index].get_name())
            {
                continue;
            }

            let array = self.fields[index].prototype_array(size);
            let output_index = output.add_field_data_array(array);
            self.fields[index].output_location = output_index as isize;

            for role in DataSetAttribute::ALL {
                if attribute_fields[role.index()] == Some(index) {
                    let _ = output.set_active_attribute_by_index_role(role, output_index as i32);
                }
            }
        }
    }

    /// VTK: `vtkDataSetAttributesFieldList::CopyData(fromId, toId)`.
    pub fn copy_data(
        &self,
        input_index: usize,
        input: &DataSetAttributes,
        from_id: usize,
        output: &mut DataSetAttributes,
        to_id: usize,
    ) -> Result<(), DataSetAttributesError> {
        for field in &self.fields {
            let Some((source_index, target_index)) =
                field.locations.get(input_index).and_then(|location| {
                    (*location >= 0 && field.output_location >= 0)
                        .then_some((*location as usize, field.output_location as usize))
                })
            else {
                continue;
            };

            let Some(source_array) = input.get_field_data_array_by_index(source_index).cloned()
            else {
                continue;
            };
            if let Some(target_array) = output.get_array_by_index_mut(target_index) {
                DataSetAttributes::copy_tuple(&source_array, target_array, from_id, to_id)?;
            }
        }
        Ok(())
    }

    /// VTK: `vtkDataSetAttributesFieldList::CopyData(inputStart, numValues, outStart)`.
    pub fn copy_data_range(
        &self,
        input_index: usize,
        input: &DataSetAttributes,
        input_start: usize,
        num_values: usize,
        output: &mut DataSetAttributes,
        out_start: usize,
    ) -> Result<(), DataSetAttributesError> {
        for offset in 0..num_values {
            self.copy_data(
                input_index,
                input,
                input_start + offset,
                output,
                out_start + offset,
            )?;
        }
        Ok(())
    }

    /// VTK: `vtkDataSetAttributesFieldList::InterpolatePoint`.
    pub fn interpolate_point(
        &self,
        input_index: usize,
        input: &DataSetAttributes,
        input_ids: &[usize],
        weights: &[f64],
        output: &mut DataSetAttributes,
        to_id: usize,
    ) -> Result<(), DataSetAttributesError> {
        for field in &self.fields {
            let Some((source_index, target_index)) =
                field.locations.get(input_index).and_then(|location| {
                    (*location >= 0 && field.output_location >= 0)
                        .then_some((*location as usize, field.output_location as usize))
                })
            else {
                continue;
            };

            let Some(source_array) = input.get_field_data_array_by_index(source_index).cloned()
            else {
                continue;
            };
            let nearest_attribute =
                field
                    .active_attribute_for_input(input_index)
                    .is_some_and(|role| {
                        output.get_copy_attribute_role(
                            role,
                            DataSetAttributeCopyOperation::Interpolate,
                        ) == 2
                    });
            if let Some(target_array) = output.get_array_by_index_mut(target_index) {
                if nearest_attribute {
                    let nearest_tuple =
                        nearest_weighted_tuple(input_ids, weights).ok_or_else(|| {
                            DataSetAttributesError::TupleOutOfRange {
                                array: source_array.get_name().to_string(),
                                tuple: 0,
                            }
                        })?;
                    if !target_array.copy_tuple_from(&source_array, nearest_tuple, to_id) {
                        return Err(DataSetAttributesError::TupleOutOfRange {
                            array: source_array.get_name().to_string(),
                            tuple: nearest_tuple,
                        });
                    }
                } else if !target_array.interpolate_tuple_from(
                    &source_array,
                    input_ids,
                    weights,
                    to_id,
                ) {
                    return Err(DataSetAttributesError::TupleOutOfRange {
                        array: source_array.get_name().to_string(),
                        tuple: input_ids.iter().copied().max().unwrap_or(0),
                    });
                }
            }
        }
        Ok(())
    }

    /// VTK: `vtkDataSetAttributesFieldList::TransformData`.
    pub fn transform_data<F>(
        &self,
        input_index: usize,
        input: &FieldData,
        output: &mut FieldData,
        mut op: F,
    ) where
        F: FnMut(&AnyArray, &mut AnyArray),
    {
        for field in &self.fields {
            let Some((source_index, target_index)) =
                field.locations.get(input_index).and_then(|location| {
                    (*location >= 0 && field.output_location >= 0)
                        .then_some((*location as usize, field.output_location as usize))
                })
            else {
                continue;
            };

            let Some(source_array) = input.get_field_data_array_by_index(source_index) else {
                continue;
            };
            if let Some(target_array) = output.arrays_mut().get_mut(target_index) {
                op(source_array.get_data(), target_array.get_data_mut());
            }
        }
    }

    /// VTK: `vtkDataSetAttributesFieldList::BuildPrototype`.
    pub fn build_prototype(
        &self,
        prototype: &mut DataSetAttributes,
        ordering: Option<&DataSetAttributes>,
    ) {
        let ordered_fields: Vec<&FieldInfo> = if let Some(ordering) = ordering {
            ordering
                .iter()
                .filter_map(|array| {
                    self.fields
                        .iter()
                        .find(|field| field.name == array.get_name())
                })
                .collect()
        } else {
            self.fields.iter().collect()
        };

        for field in ordered_fields {
            let index = prototype.add_field_data_array(field.prototype_array(0));
            for role in DataSetAttribute::ALL {
                if field
                    .attribute_types
                    .first()
                    .is_some_and(|attrs| attrs[role.index()])
                {
                    let _ = prototype.set_active_attribute_by_index_role(role, index as i32);
                    break;
                }
            }
        }
    }

    /// VTK: `vtkDataSetAttributesFieldList::GetNumberOfArrays`.
    pub fn get_number_of_arrays(&self) -> usize {
        self.fields.len()
    }

    #[cfg(test)]
    pub(crate) fn fields(&self) -> &[FieldInfo] {
        &self.fields
    }

    /// VTK: `vtkDataSetAttributesFieldList::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = String::from("vtkDataSetAttributesFieldList\n");
        out.push_str(&format!("NumberOfInputs: {}\n", self.number_of_inputs));
        out.push_str(&format!("NumberOfTuples: {}\n", self.number_of_tuples));
        out.push_str(&format!("NumberOfArrays: {}\n", self.fields.len()));
        for field in &self.fields {
            out.push_str(&format!(
                "FieldInfo {{ name: {}, components: {}, locations: {:?}, output_location: {} }}\n",
                field.name, field.number_of_components, field.locations, field.output_location
            ));
        }
        out
    }

    fn get_fields(dsa: &DataSetAttributes) -> Vec<FieldInfo> {
        let attrs = dsa.get_attribute_indices();
        dsa.iter()
            .enumerate()
            .map(|(index, array)| FieldInfo::create(array, index, &attrs))
            .collect()
    }

    fn attribute_field_indices(&self) -> [Option<usize>; DataSetAttribute::ALL.len()] {
        let mut attrs = [None; DataSetAttribute::ALL.len()];
        for (index, field) in self.fields.iter().enumerate() {
            for role in DataSetAttribute::ALL {
                if attrs[role.index()].is_none() && field.consistently_active_for(role) {
                    attrs[role.index()] = Some(index);
                }
            }
        }
        attrs
    }
}

fn names_in_both(left: &[FieldInfo], right: &[FieldInfo]) -> Vec<String> {
    let left_names: BTreeSet<_> = left.iter().map(|field| field.name.clone()).collect();
    let right_names: BTreeSet<_> = right.iter().map(|field| field.name.clone()).collect();
    left_names.intersection(&right_names).cloned().collect()
}

fn nearest_weighted_tuple(input_ids: &[usize], weights: &[f64]) -> Option<usize> {
    let mut nearest = *input_ids.first()?;
    let mut max_weight = 0.0;
    for (&input_id, &weight) in input_ids.iter().zip(weights) {
        if weight > max_weight {
            max_weight = weight;
            nearest = input_id;
        }
    }
    Some(nearest)
}
