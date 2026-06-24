use thiserror::Error;

#[derive(Debug, Error)]
pub enum VtkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("unsupported feature: {0}")]
    Unsupported(String),

    #[error("index out of bounds: {index} (size: {size})")]
    IndexOutOfBounds { index: usize, size: usize },

    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: String, got: String },

    #[error("empty data: {0}")]
    EmptyData(String),
}

impl VtkError {
    /// Create an index-out-of-bounds error.
    pub fn index_oob(index: usize, size: usize) -> Self {
        Self::IndexOutOfBounds { index, size }
    }

    /// Create a dimension mismatch error.
    pub fn dim_mismatch(expected: impl Into<String>, got: impl Into<String>) -> Self {
        Self::DimensionMismatch {
            expected: expected.into(),
            got: got.into(),
        }
    }

    /// Add context to any VtkError variant.
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        match self {
            VtkError::Parse(msg) => VtkError::Parse(format!("{ctx}: {msg}")),
            VtkError::InvalidData(msg) => VtkError::InvalidData(format!("{ctx}: {msg}")),
            VtkError::Unsupported(msg) => VtkError::Unsupported(format!("{ctx}: {msg}")),
            VtkError::EmptyData(msg) => VtkError::EmptyData(format!("{ctx}: {msg}")),
            other => other,
        }
    }

    /// Check if this is an I/O error.
    pub fn is_io(&self) -> bool {
        matches!(self, VtkError::Io(_))
    }

    /// Check if this is a parse error.
    pub fn is_parse(&self) -> bool {
        matches!(self, VtkError::Parse(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = VtkError::Parse("bad input".into());
        assert_eq!(format!("{e}"), "parse error: bad input");
    }

    #[test]
    fn index_oob() {
        let e = VtkError::index_oob(5, 3);
        let msg = format!("{e}");
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn dim_mismatch() {
        let e = VtkError::dim_mismatch("3x3", "4x4");
        assert!(format!("{e}").contains("3x3"));
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let vtk_err: VtkError = io_err.into();
        assert!(matches!(vtk_err, VtkError::Io(_)));
        assert!(vtk_err.is_io());
    }

    #[test]
    fn with_context() {
        let e = VtkError::Parse("bad token".into()).with_context("reading file.vtk");
        let msg = format!("{e}");
        assert!(msg.contains("reading file.vtk"));
        assert!(msg.contains("bad token"));
    }

    #[test]
    fn is_parse() {
        assert!(VtkError::Parse("x".into()).is_parse());
        assert!(!VtkError::InvalidData("x".into()).is_parse());
    }
}
