use std::sync::OnceLock;

use crate::common::core::InformationRequestKey;

static REQUEST_INFORMATION_KEY: OnceLock<usize> = OnceLock::new();

/// VTK: `vtkDemandDrivenPipeline`.
#[derive(Debug)]
pub struct DemandDrivenPipeline;

impl DemandDrivenPipeline {
    /// VTK: `vtkDemandDrivenPipeline::REQUEST_INFORMATION`.
    pub fn request_information() -> &'static InformationRequestKey {
        let key = *REQUEST_INFORMATION_KEY.get_or_init(|| {
            InformationRequestKey::make_key(
                Some("REQUEST_INFORMATION"),
                Some("vtkDemandDrivenPipeline"),
            ) as usize
        });
        unsafe { &*(key as *const InformationRequestKey) }
    }
}
