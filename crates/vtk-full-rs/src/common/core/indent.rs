use std::fmt;

/// VTK: `VTK_STD_INDENT`.
pub const VTK_STD_INDENT: i32 = 2;

/// VTK: `VTK_NUMBER_OF_BLANKS`.
pub const VTK_NUMBER_OF_BLANKS: i32 = 40;

/// VTK: `vtkIndent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indent {
    indent: i32,
}

impl Default for Indent {
    fn default() -> Self {
        Self::new()
    }
}

impl Indent {
    /// VTK: `vtkIndent::New` and `vtkIndent::vtkIndent(int ind = 0)`.
    pub fn new() -> Self {
        Self { indent: 0 }
    }

    /// VTK: `vtkIndent::vtkIndent(int ind = 0)`.
    pub fn new_with_indent(indent: i32) -> Self {
        Self { indent }
    }

    /// VTK: `vtkIndent::Delete`.
    pub fn delete(self) {}

    /// VTK: `vtkIndent::GetNextIndent`.
    pub fn get_next_indent(&self) -> Self {
        Self {
            indent: (self.indent + VTK_STD_INDENT).min(VTK_NUMBER_OF_BLANKS),
        }
    }
}

/// VTK: `operator<<(ostream&, const vtkIndent&)`.
impl fmt::Display for Indent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.indent.clamp(0, VTK_NUMBER_OF_BLANKS) as usize;
        for _ in 0..count {
            formatter.write_str(" ")?;
        }
        Ok(())
    }
}
