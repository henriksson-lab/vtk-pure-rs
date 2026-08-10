use std::{fmt, ops::Deref};

/// VTK: `vtkStdString`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StdString {
    string: String,
}

impl StdString {
    /// VTK: `vtkStdString()`.
    pub fn new() -> Self {
        Self {
            string: String::new(),
        }
    }

    /// VTK: `vtkStdString(const value_type* s)`.
    pub fn new_from_str(value: &str) -> Self {
        Self {
            string: value.to_string(),
        }
    }

    /// VTK: `vtkStdString(const value_type* s, size_type n)`.
    pub fn new_from_str_with_len(value: &str, len: usize) -> Self {
        let end = len.min(value.len());
        let prefix = value
            .get(..end)
            .expect("vtkStdString byte length must end on a UTF-8 boundary");
        Self::new_from_str(prefix)
    }

    /// VTK: `vtkStdString(const std::string& s)`.
    pub fn new_from_string(value: &String) -> Self {
        Self {
            string: value.clone(),
        }
    }

    /// VTK: `vtkStdString(std::string&& s)`.
    pub fn new_from_string_move(value: String) -> Self {
        Self { string: value }
    }

    /// VTK: `vtkStdString(const std::string& s, size_type pos, size_type n)`.
    pub fn new_from_string_range(value: &str, pos: usize, len: usize) -> Self {
        let end = pos.saturating_add(len).min(value.len());
        let substring = value
            .get(pos..end)
            .expect("vtkStdString range must use UTF-8 boundaries");
        Self::new_from_str(substring)
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn into_string(self) -> String {
        self.string
    }
}

impl From<&str> for StdString {
    fn from(value: &str) -> Self {
        Self::new_from_str(value)
    }
}

impl From<String> for StdString {
    fn from(value: String) -> Self {
        Self::new_from_string_move(value)
    }
}

impl From<StdString> for String {
    fn from(value: StdString) -> Self {
        value.into_string()
    }
}

impl AsRef<str> for StdString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for StdString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// VTK: `operator<<(ostream&, const vtkStdString&)`.
impl fmt::Display for StdString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
