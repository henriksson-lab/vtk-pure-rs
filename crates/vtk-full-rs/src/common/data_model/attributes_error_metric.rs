use crate::common::core::{TimeStamp, VtkMTimeType};

use super::{
    GenericAttributeCollectionHandle, GenericSubdivisionErrorMetric,
    GenericSubdivisionErrorMetricApi,
};

const ATTRIBUTE_OFFSET: usize = 6;

/// VTK: `vtkAttributesErrorMetric`.
#[derive(Debug, Clone)]
pub struct AttributesErrorMetric {
    metric: GenericSubdivisionErrorMetric,
    attribute_tolerance: f64,
    square_absolute_attribute_tolerance: f64,
    absolute_attribute_tolerance: f64,
    defined_by_absolute: i32,
    square_absolute_attribute_tolerance_compute_time: TimeStamp,
    range: f64,
}

impl AttributesErrorMetric {
    /// VTK: `vtkAttributesErrorMetric::New`.
    pub fn new() -> Self {
        let absolute_attribute_tolerance = 0.1;
        Self {
            metric: GenericSubdivisionErrorMetric::with_class_name("vtkAttributesErrorMetric"),
            attribute_tolerance: 0.1,
            square_absolute_attribute_tolerance: absolute_attribute_tolerance
                * absolute_attribute_tolerance,
            absolute_attribute_tolerance,
            defined_by_absolute: 1,
            square_absolute_attribute_tolerance_compute_time: TimeStamp::new(),
            range: 0.0,
        }
    }

    /// VTK: `vtkAttributesErrorMetric::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.metric.print_self();
        result.push_str(&format!(
            "AttributeTolerance: {}\n",
            self.attribute_tolerance
        ));
        result.push_str(&format!(
            "AbsoluteAttributeTolerance: {}\n",
            self.absolute_attribute_tolerance
        ));
        result
    }

    /// VTK: `vtkAttributesErrorMetric::GetAbsoluteAttributeTolerance`.
    pub fn get_absolute_attribute_tolerance(&self) -> f64 {
        self.absolute_attribute_tolerance
    }

    /// VTK: `vtkAttributesErrorMetric::SetAbsoluteAttributeTolerance`.
    pub fn set_absolute_attribute_tolerance(&mut self, value: f64) {
        assert!(value > 0.0, "pre: valid_range_value");
        if self.absolute_attribute_tolerance != value || self.defined_by_absolute == 0 {
            self.absolute_attribute_tolerance = value;
            self.square_absolute_attribute_tolerance =
                self.absolute_attribute_tolerance * self.absolute_attribute_tolerance;
            self.range = 0.0;
            self.defined_by_absolute = 1;
            self.modified();
        }
    }

    /// VTK: `vtkAttributesErrorMetric::GetAttributeTolerance`.
    pub fn get_attribute_tolerance(&self) -> f64 {
        self.attribute_tolerance
    }

    /// VTK: `vtkAttributesErrorMetric::SetAttributeTolerance`.
    pub fn set_attribute_tolerance(&mut self, value: f64) {
        assert!(value > 0.0 && value < 1.0, "pre: valid_range_value");
        if self.attribute_tolerance != value || self.defined_by_absolute != 0 {
            self.attribute_tolerance = value;
            self.defined_by_absolute = 0;
            self.modified();
        }
    }

    /// VTK: `vtkAttributesErrorMetric::ComputeSquareAbsoluteAttributeTolerance`.
    fn compute_square_absolute_attribute_tolerance(&mut self) {
        if self.defined_by_absolute != 0
            || self.get_m_time()
                <= self
                    .square_absolute_attribute_tolerance_compute_time
                    .get_m_time()
        {
            return;
        }

        let attributes = self.attributes();
        let attributes_ref = attributes.borrow();
        let active_attribute = attributes_ref.get_active_attribute();
        let active_component = attributes_ref.get_active_component();
        let attribute = attributes_ref
            .get_attribute(active_attribute)
            .expect("pre: not_empty");
        drop(attributes_ref);

        let mut range = [0.0, 0.0];
        attribute
            .borrow()
            .get_range_into(active_component, &mut range);

        let delta = range[1] - range[0];
        let absolute = delta * self.attribute_tolerance;
        self.range = delta;
        self.square_absolute_attribute_tolerance = absolute * absolute;
        self.square_absolute_attribute_tolerance_compute_time
            .modified();
        self.absolute_attribute_tolerance = self.square_absolute_attribute_tolerance.sqrt();
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetGenericCell`.
    pub fn set_generic_cell(&mut self, cell: Option<super::GenericAdaptorCellHandle>) {
        self.metric.set_generic_cell(cell);
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetGenericCell`.
    pub fn get_generic_cell(&self) -> Option<super::GenericAdaptorCellHandle> {
        self.metric.get_generic_cell()
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: Option<super::GenericDataSetHandle>) {
        self.metric.set_data_set(data_set);
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetDataSet`.
    pub fn get_data_set(&self) -> Option<super::GenericDataSetHandle> {
        self.metric.get_data_set()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.metric.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.metric.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.metric.get_m_time()
    }

    fn attributes(&self) -> GenericAttributeCollectionHandle {
        self.metric
            .get_data_set()
            .expect("pre: dataset_exists")
            .borrow()
            .get_attributes()
            .expect("pre: attribute_collection_exists")
    }

    fn attribute_error(
        &self,
        attributes: GenericAttributeCollectionHandle,
        left_point: &[f64],
        mid_point: &[f64],
        right_point: &[f64],
        alpha: f64,
    ) -> f64 {
        let attributes_ref = attributes.borrow();
        let active_attribute = attributes_ref.get_active_attribute();
        let active_component = attributes_ref.get_active_component();
        let attribute = attributes_ref
            .get_attribute(active_attribute)
            .expect("pre: not_empty");

        let generic_cell = self
            .metric
            .get_generic_cell()
            .expect("pre: generic_cell_exists");
        if generic_cell.borrow().is_attribute_linear(attribute) {
            return 0.0;
        }

        let attribute_index =
            attributes_ref.get_attribute_index(active_attribute) as usize + ATTRIBUTE_OFFSET;
        if active_component >= 0 {
            let i = attribute_index + active_component as usize;
            assert!(i < left_point.len(), "pre: valid_size");
            assert!(i < mid_point.len(), "pre: valid_size");
            assert!(i < right_point.len(), "pre: valid_size");
            let delta = left_point[i] + alpha * (right_point[i] - left_point[i]) - mid_point[i];
            delta * delta
        } else {
            let components = attributes_ref.get_number_of_components() as usize;
            let mut ae = 0.0;
            for j in 0..components {
                let i = attribute_index + j;
                assert!(i < left_point.len(), "pre: valid_size");
                assert!(i < mid_point.len(), "pre: valid_size");
                assert!(i < right_point.len(), "pre: valid_size");
                let delta = left_point[i] + alpha * (right_point[i] - left_point[i]) - mid_point[i];
                ae += delta * delta;
            }
            ae
        }
    }
}

impl Default for AttributesErrorMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericSubdivisionErrorMetricApi for AttributesErrorMetric {
    fn set_generic_cell(&mut self, cell: Option<super::GenericAdaptorCellHandle>) {
        self.set_generic_cell(cell);
    }

    fn get_generic_cell(&self) -> Option<super::GenericAdaptorCellHandle> {
        self.get_generic_cell()
    }

    fn set_data_set(&mut self, data_set: Option<super::GenericDataSetHandle>) {
        self.set_data_set(data_set);
    }

    fn get_data_set(&self) -> Option<super::GenericDataSetHandle> {
        self.get_data_set()
    }

    fn requires_edge_subdivision(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) -> i32 {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        assert!(alpha > 0.0 && alpha < 1.0, "pre: clamped_alpha");

        self.compute_square_absolute_attribute_tolerance();
        let attributes = self.attributes();
        let ae = self.attribute_error(attributes, left_point, mid_point, right_point, alpha);

        if self.square_absolute_attribute_tolerance == 0.0 {
            (ae.abs() > 0.0001) as i32
        } else {
            (ae > self.square_absolute_attribute_tolerance) as i32
        }
    }

    fn get_error(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) -> f64 {
        assert!(!left_point.is_empty(), "pre: leftPoint_exists");
        assert!(!mid_point.is_empty(), "pre: midPoint_exists");
        assert!(!right_point.is_empty(), "pre: rightPoint_exists");
        assert!(alpha > 0.0 && alpha < 1.0, "pre: clamped_alpha");

        self.compute_square_absolute_attribute_tolerance();
        let attributes = self.attributes();
        let ae = self.attribute_error(attributes, left_point, mid_point, right_point, alpha);

        if self.range != 0.0 {
            ae.sqrt() / self.range
        } else {
            0.0
        }
    }
}
