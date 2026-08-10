use std::time::Duration;

/// VTK: `vtkBreakPoint`.
pub struct BreakPoint;

impl BreakPoint {
    /// VTK: `vtkBreakPoint::Break`.
    #[cfg(not(windows))]
    pub fn r#break() {
        let i = 0;
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        println!(
            "PID {} on {} ready for attach",
            std::process::id(),
            hostname
        );
        while i == 0 {
            std::thread::sleep(Duration::from_secs(5));
        }
    }

    /// VTK: `vtkBreakPoint::Break`.
    #[cfg(windows)]
    pub fn r#break() {}
}
