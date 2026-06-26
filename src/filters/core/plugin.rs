//! Plugin registry for dynamically registering filters and readers.
//!
//! Uses a simple in-process registry pattern (no dynamic loading) so that
//! users can register custom filter and reader functions at runtime.

use crate::data::PolyData;
use std::collections::HashMap;
use std::path::Path;

/// Registry of named filter functions that transform a `PolyData`.
pub struct FilterRegistry {
    filters: HashMap<String, Box<dyn Fn(&PolyData) -> PolyData + Send + Sync>>,
}

impl FilterRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
        }
    }

    /// Register a filter function under the given name.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        f: impl Fn(&PolyData) -> PolyData + Send + Sync + 'static,
    ) {
        self.filters.insert(name.into(), Box::new(f));
    }

    /// Apply a registered filter by name. Returns `None` if the name is unknown.
    pub fn apply(&self, name: &str, data: &PolyData) -> Option<PolyData> {
        self.filters.get(name).map(|f| f(data))
    }

    /// List all registered filter names.
    pub fn list(&self) -> Vec<&str> {
        self.filters.keys().map(|s| s.as_str()).collect()
    }

    /// Check whether a filter with the given name is registered.
    pub fn has(&self, name: &str) -> bool {
        self.filters.contains_key(name)
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry of reader functions keyed by file extension.
pub struct ReaderRegistry {
    readers: HashMap<String, Box<dyn Fn(&Path) -> Result<PolyData, String> + Send + Sync>>,
}

impl ReaderRegistry {
    /// Create an empty reader registry.
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
        }
    }

    /// Register a reader for files with the given extension (without the dot).
    pub fn register(
        &mut self,
        extension: impl Into<String>,
        reader: impl Fn(&Path) -> Result<PolyData, String> + Send + Sync + 'static,
    ) {
        self.readers
            .insert(extension.into().to_lowercase(), Box::new(reader));
    }

    /// Read a file, choosing the reader by file extension.
    pub fn read(&self, path: &Path) -> Result<PolyData, String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .ok_or_else(|| "no file extension".to_string())?;
        let reader = self
            .readers
            .get(&ext)
            .ok_or_else(|| format!("no reader registered for extension '{ext}'"))?;
        reader(path)
    }

    /// List all registered extensions.
    pub fn list(&self) -> Vec<&str> {
        self.readers.keys().map(|s| s.as_str()).collect()
    }

    /// Check whether a reader for the given extension exists.
    pub fn has(&self, extension: &str) -> bool {
        self.readers.contains_key(&extension.to_lowercase())
    }
}

impl Default for ReaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_registry_roundtrip() {
        let mut reg = FilterRegistry::new();
        assert!(!reg.has("identity"));

        reg.register("identity", |pd| pd.clone());
        assert!(reg.has("identity"));
        assert!(reg.list().contains(&"identity"));

        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = reg.apply("identity", &mesh).unwrap();
        assert_eq!(result.points.len(), 3);
        assert!(reg.apply("nonexistent", &mesh).is_none());
    }

    #[test]
    fn reader_registry() {
        let mut reg = ReaderRegistry::new();
        reg.register("xyz", |_path| {
            Ok(PolyData::from_points(vec![[1.0, 2.0, 3.0]]))
        });
        assert!(reg.has("xyz"));
        assert!(reg.has("XYZ"));
        assert!(!reg.has("abc"));

        let result = reg.read(Path::new("/tmp/test.xyz"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().points.len(), 1);

        let result = reg.read(Path::new("/tmp/test.XYZ"));
        assert!(result.is_ok());

        let err = reg.read(Path::new("/tmp/test.abc"));
        assert!(err.is_err());
    }
}
