use crate::common::core::{DoubleArray, Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkContourValues`.
#[derive(Debug, Clone)]
pub struct ContourValues {
    object: Object,
    contours: Option<DoubleArray>,
}

impl ContourValues {
    /// VTK: `vtkContourValues::New`.
    pub fn new() -> Self {
        let mut contours = DoubleArray::new();
        contours.reserve_values(64);
        contours.insert_tuple1(0, 0.0);
        Self {
            object: Object::with_class_name("vtkContourValues"),
            contours: Some(contours),
        }
    }

    fn contours(&self) -> &DoubleArray {
        self.contours
            .as_ref()
            .expect("vtkContourValues Contours must be non-null")
    }

    fn contours_mut(&mut self) -> &mut DoubleArray {
        self.contours
            .as_mut()
            .expect("vtkContourValues Contours must be non-null")
    }

    /// VTK: `vtkContourValues::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = String::from("Contour Values: \n");
        for i in 0..self.get_number_of_contours() {
            out.push_str(&format!("  Value {}: {}\n", i, self.get_value(i)));
        }
        out
    }

    /// VTK: `vtkContourValues::SetValue`.
    pub fn set_value(&mut self, i: i32, value: f64) {
        let num_contours = self.contours().get_number_of_values();
        let index = i.max(0) as VtkIdType;

        if index >= num_contours || value != self.contours().get_tuple1(index) {
            self.modified();
            self.contours_mut().insert_tuple1(index, value);
        }
    }

    /// VTK: `vtkContourValues::GetValue`.
    pub fn get_value(&self, i: i32) -> f64 {
        let max_id = self.contours().get_number_of_values() - 1;
        let index = (i.max(0) as VtkIdType).min(max_id);
        self.contours().get_tuple1(index)
    }

    /// VTK: `vtkContourValues::GetValues()`.
    pub fn get_values(&mut self) -> *mut f64 {
        self.contours_mut().as_mut_slice().as_mut_ptr()
    }

    /// VTK: `vtkContourValues::GetValues(double*)`.
    pub fn copy_values(&self, contour_values: &mut [f64]) {
        let num_contours = self.get_number_of_contours() as usize;
        assert!(contour_values.len() >= num_contours);
        for (i, value) in contour_values.iter_mut().take(num_contours).enumerate() {
            *value = self.contours().get_tuple1(i as VtkIdType);
        }
    }

    /// VTK: `vtkContourValues::GetContours`.
    pub fn get_contours(&self) -> Option<&DoubleArray> {
        self.contours.as_ref()
    }

    /// VTK: `vtkContourValues::SetContours`.
    pub fn set_contours(&mut self, contours: Option<DoubleArray>) {
        self.contours = contours;
        self.modified();
    }

    /// VTK: `vtkContourValues::SetNumberOfContours`.
    pub fn set_number_of_contours(&mut self, number: i32) {
        let current_number = self.contours().get_number_of_values();
        let n = number.max(0) as VtkIdType;

        if n != current_number {
            self.modified();
            let old_values: Vec<_> = (0..current_number)
                .map(|i| self.contours().get_tuple1(i))
                .collect();

            self.contours_mut().set_number_of_values(n);

            let limit = current_number.min(n);
            for i in 0..limit {
                self.contours_mut().set_tuple1(i, old_values[i as usize]);
            }
        }

        if n > current_number {
            for i in current_number..n {
                self.contours_mut().set_tuple1(i, 0.0);
            }
        }
    }

    /// VTK: `vtkContourValues::GetNumberOfContours`.
    pub fn get_number_of_contours(&self) -> i32 {
        self.contours().get_number_of_values() as i32
    }

    /// VTK: `vtkContourValues::GenerateValues(int, double[2])`.
    pub fn generate_values(&mut self, num_contours: i32, range: [f64; 2]) {
        self.set_number_of_contours(num_contours);
        if num_contours == 1 {
            self.set_value(0, range[0]);
        } else {
            for i in 0..num_contours {
                self.set_value(
                    i,
                    range[0] + i as f64 * (range[1] - range[0]) / (num_contours - 1) as f64,
                );
            }
        }
    }

    /// VTK: `vtkContourValues::GenerateValues(int, double, double)`.
    pub fn generate_values_from_bounds(
        &mut self,
        num_contours: i32,
        range_start: f64,
        range_end: f64,
    ) {
        self.generate_values(num_contours, [range_start, range_end]);
    }

    /// VTK: `vtkContourValues::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.contours_mut().deep_copy(other.contours());
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkContourValues::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkContourValues" || Object::is_type_of(name)
    }

    /// VTK: `vtkContourValues::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time().max(self.contours().get_m_time())
    }
}

impl Default for ContourValues {
    fn default() -> Self {
        Self::new()
    }
}
