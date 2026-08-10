use std::{cmp::Ordering, path::Path};

use crate::common::core::{Object, StringArray, TimeStamp};

/// Container used by `vtkSortFileNames` to hold grouped string arrays.
///
/// VTK origin: anonymous `vtkStringArrayVector` helper in
/// `VTK/IO/Core/vtkSortFileNames.cxx`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct StringArrayVector {
    container: Vec<StringArray>,
}

impl StringArrayVector {
    /// VTK: `vtkStringArrayVector::New`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// VTK: `vtkStringArrayVector::Delete`.
    #[allow(dead_code)]
    pub(crate) fn delete(self) {}

    /// VTK: `vtkStringArrayVector::Reset`.
    pub(crate) fn reset(&mut self) {
        self.container.clear();
    }

    /// VTK: `vtkStringArrayVector::InsertNextStringArray`.
    pub(crate) fn insert_next_string_array(&mut self, string_array: &StringArray) {
        self.container.push(string_array.shallow_clone());
    }

    /// VTK: `vtkStringArrayVector::GetStringArray`.
    pub(crate) fn get_string_array(&self, i: i32) -> Option<&StringArray> {
        usize::try_from(i)
            .ok()
            .and_then(|index| self.container.get(index))
    }

    /// VTK: `vtkStringArrayVector::GetNumberOfStringArrays`.
    pub(crate) fn get_number_of_string_arrays(&self) -> i32 {
        i32::try_from(self.container.len()).expect("group count must fit i32")
    }
}

/// VTK: `vtkSortFileNames`.
#[derive(Debug, Clone, PartialEq)]
pub struct SortFileNames {
    object: Object,
    numeric_sort: bool,
    ignore_case: bool,
    grouping: bool,
    skip_directories: bool,
    update_time: TimeStamp,
    input_file_names: Option<StringArray>,
    file_names: StringArray,
    groups: StringArrayVector,
}

impl SortFileNames {
    /// VTK: `vtkSortFileNames::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSortFileNames"),
            numeric_sort: false,
            ignore_case: false,
            grouping: false,
            skip_directories: false,
            update_time: TimeStamp::new(),
            input_file_names: None,
            file_names: StringArray::new(),
            groups: StringArrayVector::new(),
        }
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
    pub fn get_m_time(&self) -> u64 {
        self.object.get_m_time()
    }

    /// VTK: `vtkSortFileNames::SetNumericSort`.
    pub fn set_numeric_sort(&mut self, value: bool) {
        if self.numeric_sort != value {
            self.numeric_sort = value;
            self.modified();
        }
    }

    /// VTK: `vtkSortFileNames::GetNumericSort`.
    pub fn get_numeric_sort(&self) -> bool {
        self.numeric_sort
    }

    /// VTK: `vtkSortFileNames::NumericSortOn`.
    pub fn numeric_sort_on(&mut self) {
        self.set_numeric_sort(true);
    }

    /// VTK: `vtkSortFileNames::NumericSortOff`.
    pub fn numeric_sort_off(&mut self) {
        self.set_numeric_sort(false);
    }

    /// VTK: `vtkSortFileNames::SetIgnoreCase`.
    pub fn set_ignore_case(&mut self, value: bool) {
        if self.ignore_case != value {
            self.ignore_case = value;
            self.modified();
        }
    }

    /// VTK: `vtkSortFileNames::GetIgnoreCase`.
    pub fn get_ignore_case(&self) -> bool {
        self.ignore_case
    }

    /// VTK: `vtkSortFileNames::IgnoreCaseOn`.
    pub fn ignore_case_on(&mut self) {
        self.set_ignore_case(true);
    }

    /// VTK: `vtkSortFileNames::IgnoreCaseOff`.
    pub fn ignore_case_off(&mut self) {
        self.set_ignore_case(false);
    }

    /// VTK: `vtkSortFileNames::SetGrouping`.
    pub fn set_grouping(&mut self, value: bool) {
        if self.grouping != value {
            self.grouping = value;
            self.modified();
        }
    }

    /// VTK: `vtkSortFileNames::GetGrouping`.
    pub fn get_grouping(&self) -> bool {
        self.grouping
    }

    /// VTK: `vtkSortFileNames::GroupingOn`.
    pub fn grouping_on(&mut self) {
        self.set_grouping(true);
    }

    /// VTK: `vtkSortFileNames::GroupingOff`.
    pub fn grouping_off(&mut self) {
        self.set_grouping(false);
    }

    /// VTK: `vtkSortFileNames::SetSkipDirectories`.
    pub fn set_skip_directories(&mut self, value: bool) {
        if self.skip_directories != value {
            self.skip_directories = value;
            self.modified();
        }
    }

    /// VTK: `vtkSortFileNames::GetSkipDirectories`.
    pub fn get_skip_directories(&self) -> bool {
        self.skip_directories
    }

    /// VTK: `vtkSortFileNames::SkipDirectoriesOn`.
    pub fn skip_directories_on(&mut self) {
        self.set_skip_directories(true);
    }

    /// VTK: `vtkSortFileNames::SkipDirectoriesOff`.
    pub fn skip_directories_off(&mut self) {
        self.set_skip_directories(false);
    }

    /// VTK: `vtkSortFileNames::SetInputFileNames`.
    pub fn set_input_file_names(&mut self, input: Option<&StringArray>) {
        self.input_file_names = input.map(StringArray::shallow_clone);
        self.modified();
    }

    /// VTK: `vtkSortFileNames::GetInputFileNames`.
    pub fn get_input_file_names(&self) -> Option<&StringArray> {
        self.input_file_names.as_ref()
    }

    /// VTK: `vtkSortFileNames::GetFileNames`.
    pub fn get_file_names(&mut self) -> &StringArray {
        self.update();
        &self.file_names
    }

    /// VTK: `vtkSortFileNames::GetNumberOfGroups`.
    pub fn get_number_of_groups(&mut self) -> i32 {
        self.update();
        self.groups.get_number_of_string_arrays()
    }

    /// VTK: `vtkSortFileNames::GetNthGroup`.
    pub fn get_nth_group(&mut self, i: i32) -> Option<&StringArray> {
        self.update();

        if !self.get_grouping() {
            return None;
        }

        self.groups.get_string_array(i)
    }

    /// VTK: `vtkSortFileNames::GroupFileNames`.
    #[allow(dead_code)]
    pub(crate) fn group_file_names(&self, input: &StringArray, output: &mut StringArrayVector) {
        group_file_names(input, output, self.ignore_case);
    }

    /// VTK: `vtkSortFileNames::SortFileNames`.
    #[allow(dead_code)]
    pub(crate) fn sort_file_names(&self, input: &StringArray, output: &mut StringArray) {
        sort_file_names(
            input,
            output,
            self.ignore_case,
            self.numeric_sort,
            self.skip_directories,
        );
    }

    /// VTK: `vtkSortFileNames::Execute`.
    pub(crate) fn execute(&mut self) {
        let Some(input_file_names) = self.input_file_names.as_ref() else {
            return;
        };

        self.file_names.reset();
        sort_file_names(
            input_file_names,
            &mut self.file_names,
            self.ignore_case,
            self.numeric_sort,
            self.skip_directories,
        );

        self.groups.reset();
        if self.grouping {
            group_file_names(&self.file_names, &mut self.groups, self.ignore_case);
        }
    }

    /// VTK: `vtkSortFileNames::Update`.
    pub fn update(&mut self) {
        let Some(input_file_names) = self.input_file_names.as_ref() else {
            return;
        };

        if self.get_m_time() > self.update_time.get_m_time()
            || input_file_names.get_m_time() > self.update_time.get_m_time()
        {
            self.execute();
            self.update_time.modified();
        }
    }
}

impl Default for SortFileNames {
    fn default() -> Self {
        Self::new()
    }
}

fn group_file_names(input: &StringArray, output: &mut StringArrayVector, ignore_case: bool) {
    let mut ungrouped_files = Vec::new();
    let mut reduced_file_names = Vec::new();

    let number_of_strings = input.get_number_of_values();
    for i in 0..number_of_strings {
        let file_name = input.get_value(i);
        let (mut base_name, mut extension, file_name_path) = split_file_name(file_name);

        let numeric_extension = !extension.is_empty()
            && extension
                .as_bytes()
                .iter()
                .skip(1)
                .all(|byte| byte.is_ascii_digit());
        if numeric_extension {
            base_name.push_str(&extension);
            extension.clear();
        }

        let mut reduced_name = format!("{file_name_path}/");
        let mut in_digit_block = false;
        let mut char_block_start = 0;
        for (k, byte) in base_name.bytes().enumerate() {
            if byte.is_ascii_digit() {
                if !in_digit_block && k != 0 {
                    reduced_name.push_str(&base_name[char_block_start..k]);
                    reduced_name.push('0');
                }
                in_digit_block = true;
            } else if in_digit_block {
                char_block_start = k;
                in_digit_block = false;
            }
        }
        if !in_digit_block {
            reduced_name.push_str(&base_name[char_block_start..]);
        }
        reduced_name.push_str(&extension);

        if ignore_case {
            reduced_name = reduced_name
                .bytes()
                .map(|byte| byte.to_ascii_uppercase() as char)
                .collect();
        }

        reduced_file_names.push(reduced_name);
        ungrouped_files.push(usize::try_from(i).expect("vtkIdType must fit usize"));
    }

    while let Some(&file_index) = ungrouped_files.first() {
        let reduced_file_name = &reduced_file_names[file_index];
        let mut new_group = StringArray::new();

        let mut next_ungrouped_files = Vec::new();
        for try_index in ungrouped_files {
            if reduced_file_name == &reduced_file_names[try_index] {
                new_group.insert_next_value(
                    input.get_value(i64::try_from(try_index).expect("index must fit vtkIdType")),
                );
            } else {
                next_ungrouped_files.push(try_index);
            }
        }

        output.insert_next_string_array(&new_group);
        ungrouped_files = next_ungrouped_files;
    }
}

fn sort_file_names(
    input: &StringArray,
    output: &mut StringArray,
    ignore_case: bool,
    numeric_sort: bool,
    skip_directories: bool,
) {
    let number_of_strings = input.get_number_of_values();
    let mut file_names = Vec::new();

    for j in 0..number_of_strings {
        let file_name = input.get_value(j);
        if skip_directories && Path::new(file_name).is_dir() {
            continue;
        }
        file_names.push(file_name.to_string());
    }

    match (ignore_case, numeric_sort) {
        (true, true) => file_names.sort_by(|left, right| {
            ordering_from_less(left, right, compare_file_names_numeric_ignore_case)
        }),
        (true, false) => file_names
            .sort_by(|left, right| ordering_from_less(left, right, compare_file_names_ignore_case)),
        (false, true) => file_names
            .sort_by(|left, right| ordering_from_less(left, right, compare_file_names_numeric)),
        (false, false) => file_names.sort(),
    }

    for file_name in file_names {
        output.insert_next_value(file_name);
    }
}

fn ordering_from_less(left: &str, right: &str, less: fn(&str, &str) -> bool) -> Ordering {
    if less(left, right) {
        Ordering::Less
    } else if less(right, left) {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn split_file_name(file_name: &str) -> (String, String, String) {
    let path = Path::new(file_name);
    let file_name_path = path
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    if let Some(dot) = name.rfind('.') {
        if dot == 0 {
            return (name, String::new(), file_name_path);
        }
        (
            name[..dot].to_string(),
            name[dot..].to_string(),
            file_name_path,
        )
    } else {
        (name, String::new(), file_name_path)
    }
}

fn compare_file_names_ignore_case(s1: &str, s2: &str) -> bool {
    let n1 = s1.len();
    let n2 = s2.len();
    let n = n1.min(n2);

    for i in 0..n {
        let c1 = s1.as_bytes()[i].to_ascii_uppercase();
        let c2 = s2.as_bytes()[i].to_ascii_uppercase();

        if c1 < c2 {
            return true;
        }
        if c1 > c2 {
            return false;
        }
    }

    if n1 < n2 {
        return true;
    }

    if n1 == n2 {
        return s1 < s2;
    }

    false
}

fn compare_file_names_numeric(s1: &str, s2: &str) -> bool {
    compare_file_names_numeric_with_case(s1, s2, false)
}

fn compare_file_names_numeric_ignore_case(s1: &str, s2: &str) -> bool {
    compare_file_names_numeric_with_case(s1, s2, true)
}

fn compare_file_names_numeric_with_case(s1: &str, s2: &str, ignore_case: bool) -> bool {
    let n1 = s1.len();
    let n2 = s2.len();
    let b1 = s1.as_bytes();
    let b2 = s2.as_bytes();

    let mut i1 = 0;
    let mut i2 = 0;
    while i1 < n1 && i2 < n2 {
        let mut c1 = b1[i1];
        let mut c2 = b2[i2];
        i1 += 1;
        i2 += 1;

        if c1.is_ascii_digit() && c2.is_ascii_digit() {
            let mut j1 = 0u32;
            while c1.is_ascii_digit() {
                j1 = j1
                    .wrapping_shl(3)
                    .wrapping_add(j1.wrapping_shl(1))
                    .wrapping_add(u32::from(c1 - b'0'));
                if i1 == n1 {
                    break;
                }
                c1 = b1[i1];
                i1 += 1;
            }

            let mut j2 = 0u32;
            while c2.is_ascii_digit() {
                j2 = j2
                    .wrapping_shl(3)
                    .wrapping_add(j2.wrapping_shl(1))
                    .wrapping_add(u32::from(c2 - b'0'));
                if i2 == n2 {
                    break;
                }
                c2 = b2[i2];
                i2 += 1;
            }

            if j1 < j2 {
                return true;
            }
            if j1 > j2 {
                return false;
            }
        }

        if !c1.is_ascii_digit() || !c2.is_ascii_digit() {
            if ignore_case {
                c1 = c1.to_ascii_uppercase();
                c2 = c2.to_ascii_uppercase();
            }

            if c1 < c2 {
                return true;
            }
            if c1 > c2 {
                return false;
            }
        }
    }

    if (n1 - i1) < (n2 - i2) {
        return true;
    }

    if i1 == n1 && i2 == n2 {
        if ignore_case {
            compare_file_names_ignore_case(s1, s2)
        } else {
            s1 < s2
        }
    } else {
        false
    }
}
