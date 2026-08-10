use std::sync::OnceLock;

use crate::common::core::{
    InformationDoubleKey, InformationDoubleVectorKey, InformationRequestKey,
};

static REQUEST_UPDATE_EXTENT_KEY: OnceLock<usize> = OnceLock::new();
static TIME_STEPS_KEY: OnceLock<usize> = OnceLock::new();
static TIME_RANGE_KEY: OnceLock<usize> = OnceLock::new();
static UPDATE_TIME_STEP_KEY: OnceLock<usize> = OnceLock::new();

/// VTK: `vtkStreamingDemandDrivenPipeline`.
#[derive(Debug)]
pub struct StreamingDemandDrivenPipeline;

impl StreamingDemandDrivenPipeline {
    /// VTK: `vtkStreamingDemandDrivenPipeline::REQUEST_UPDATE_EXTENT`.
    pub fn request_update_extent() -> &'static InformationRequestKey {
        let key = *REQUEST_UPDATE_EXTENT_KEY.get_or_init(|| {
            InformationRequestKey::make_key(
                Some("REQUEST_UPDATE_EXTENT"),
                Some("vtkStreamingDemandDrivenPipeline"),
            ) as usize
        });
        unsafe { &*(key as *const InformationRequestKey) }
    }

    /// VTK: `vtkStreamingDemandDrivenPipeline::TIME_STEPS`.
    pub fn time_steps() -> &'static InformationDoubleVectorKey {
        let key = *TIME_STEPS_KEY.get_or_init(|| {
            InformationDoubleVectorKey::make_key(
                Some("TIME_STEPS"),
                Some("vtkStreamingDemandDrivenPipeline"),
                -1,
            ) as usize
        });
        unsafe { &*(key as *const InformationDoubleVectorKey) }
    }

    /// VTK: `vtkStreamingDemandDrivenPipeline::TIME_RANGE`.
    pub fn time_range() -> &'static InformationDoubleVectorKey {
        let key = *TIME_RANGE_KEY.get_or_init(|| {
            InformationDoubleVectorKey::make_key(
                Some("TIME_RANGE"),
                Some("vtkStreamingDemandDrivenPipeline"),
                2,
            ) as usize
        });
        unsafe { &*(key as *const InformationDoubleVectorKey) }
    }

    /// VTK: `vtkStreamingDemandDrivenPipeline::UPDATE_TIME_STEP`.
    pub fn update_time_step() -> &'static InformationDoubleKey {
        let key = *UPDATE_TIME_STEP_KEY.get_or_init(|| {
            InformationDoubleKey::make_key(
                Some("UPDATE_TIME_STEP"),
                Some("vtkStreamingDemandDrivenPipeline"),
            ) as usize
        });
        unsafe { &*(key as *const InformationDoubleKey) }
    }
}
