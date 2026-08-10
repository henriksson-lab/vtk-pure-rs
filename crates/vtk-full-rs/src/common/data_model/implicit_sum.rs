use std::fmt;

use crate::common::core::{Object, VtkMTimeType};
use crate::common::data_model::{ImplicitFunctionCollection, ImplicitFunctionHandle};

/// VTK: `vtkImplicitSum`.
#[derive(Clone)]
pub struct ImplicitSum {
    object: Object,
    function_list: ImplicitFunctionCollection,
    weights: Vec<f64>,
    weights_object: Object,
    total_weight: f64,
    normalize_by_weight: bool,
}

impl ImplicitSum {
    /// VTK: `vtkImplicitSum::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkImplicitSum"),
            function_list: ImplicitFunctionCollection::new(),
            weights: Vec::new(),
            weights_object: Object::with_class_name("vtkDoubleArray"),
            total_weight: 0.0,
            normalize_by_weight: false,
        }
    }

    /// VTK: `vtkImplicitSum::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.function_list.iter().fold(
            self.object
                .get_m_time()
                .max(self.weights_object.get_m_time()),
            |mtime, function| mtime.max(function.get_m_time()),
        )
    }

    /// VTK: `vtkImplicitSum::AddFunction`.
    pub fn add_function(&mut self, function: ImplicitFunctionHandle, weight: f64) {
        self.modified();
        self.function_list.add_item(function);
        self.weights.push(weight);
        self.weights_object.modified();
        self.calculate_total_weight();
    }

    /// VTK: `vtkImplicitSum::SetFunctionWeight`.
    pub fn set_function_weight(&mut self, function: &ImplicitFunctionHandle, weight: f64) {
        let index = self.function_list.index_of_first_occurrence(function);
        if index < 0 {
            return;
        }
        let index = index as usize;

        if self.weights[index] != weight {
            self.modified();
            self.weights[index] = weight;
            self.weights_object.modified();
            self.calculate_total_weight();
        }
    }

    /// VTK: `vtkImplicitSum::RemoveAllFunctions`.
    pub fn remove_all_functions(&mut self) {
        self.modified();
        self.function_list.remove_all_items();
        self.weights.clear();
        self.weights_object.modified();
        self.total_weight = 0.0;
    }

    /// VTK: `vtkImplicitSum::CalculateTotalWeight`.
    pub(crate) fn calculate_total_weight(&mut self) {
        self.total_weight = self.weights.iter().sum();
    }

    /// VTK: `vtkImplicitSum::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let mut sum = 0.0;
        for (function, weight) in self.function_list.iter().zip(&self.weights) {
            if *weight != 0.0 {
                sum += function.evaluate_function(x) * *weight;
            }
        }
        if self.normalize_by_weight && self.total_weight != 0.0 {
            sum / self.total_weight
        } else {
            sum
        }
    }

    /// VTK: `vtkImplicitSum::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        let mut gradient = [0.0; 3];
        for (function, weight) in self.function_list.iter().zip(&self.weights) {
            if *weight != 0.0 {
                let component_gradient = function.evaluate_gradient(x);
                gradient[0] += component_gradient[0] * *weight;
                gradient[1] += component_gradient[1] * *weight;
                gradient[2] += component_gradient[2] * *weight;
            }
        }

        if self.normalize_by_weight && self.total_weight != 0.0 {
            gradient[0] /= self.total_weight;
            gradient[1] /= self.total_weight;
            gradient[2] /= self.total_weight;
        }
        gradient
    }

    /// VTK: `vtkImplicitSum::SetNormalizeByWeight`.
    pub fn set_normalize_by_weight(&mut self, normalize_by_weight: bool) {
        if self.normalize_by_weight != normalize_by_weight {
            self.normalize_by_weight = normalize_by_weight;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitSum::GetNormalizeByWeight`.
    pub fn get_normalize_by_weight(&self) -> bool {
        self.normalize_by_weight
    }

    /// VTK: `vtkImplicitSum::NormalizeByWeightOn`.
    pub fn normalize_by_weight_on(&mut self) {
        self.set_normalize_by_weight(true);
    }

    /// VTK: `vtkImplicitSum::NormalizeByWeightOff`.
    pub fn normalize_by_weight_off(&mut self) {
        self.set_normalize_by_weight(false);
    }

    /// VTK: `vtkImplicitSum::PrintSelf`.
    pub fn print_self(&self) -> String {
        let normalize = if self.normalize_by_weight {
            "On"
        } else {
            "Off"
        };
        format!(
            "NormalizeByWeight: {normalize}\nFunction List: {} items\nWeights: {:?}\n",
            self.function_list.len(),
            self.weights
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }
}

impl Default for ImplicitSum {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ImplicitSum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitSum")
            .field("class_name", &self.get_class_name())
            .field("function_count", &self.function_list.len())
            .field("weights", &self.weights)
            .field("total_weight", &self.total_weight)
            .field("normalize_by_weight", &self.normalize_by_weight)
            .finish()
    }
}
