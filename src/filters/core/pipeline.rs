use crate::data::PolyData;

/// A filter function that transforms PolyData.
pub type FilterFn = Box<dyn Fn(&PolyData) -> PolyData + Send + Sync>;

/// A pipeline stage with a cached output.
struct Stage {
    name: String,
    filter: FilterFn,
    cache: Option<PolyData>,
    dirty: bool,
}

/// A pipeline of chained filters with lazy evaluation and caching.
///
/// Filters are applied sequentially: input → filter1 → filter2 → ... → output.
/// Results are cached. When a stage is invalidated (marked dirty), all
/// downstream stages are also invalidated.
///
/// ```
/// use crate::data::PolyData;
/// use crate::filters::core::pipeline::Pipeline;
///
/// let pd = PolyData::from_triangles(
///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
///     vec![[0, 1, 2]],
/// );
/// let mut pipe = Pipeline::new(pd)
///     .with_normals()
///     .with_elevation_z();
/// let result = pipe.output();
/// assert!(result.point_data().normals().is_some());
/// ```
pub struct Pipeline {
    input: PolyData,
    stages: Vec<Stage>,
    input_dirty: bool,
}

impl Pipeline {
    /// Create a pipeline with the given input data.
    pub fn new(input: PolyData) -> Self {
        Self {
            input,
            stages: Vec::new(),
            input_dirty: true,
        }
    }

    /// Add a filter stage to the end of the pipeline.
    pub fn add(&mut self, name: &str, filter: FilterFn) {
        self.stages.push(Stage {
            name: name.to_string(),
            filter,
            cache: None,
            dirty: true,
        });
    }

    /// Set new input data. Invalidates all stages.
    pub fn set_input(&mut self, input: PolyData) {
        self.input = input;
        self.input_dirty = true;
        self.invalidate_from(0);
    }

    /// Invalidate a stage by name. All downstream stages are also invalidated.
    pub fn invalidate(&mut self, name: &str) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.invalidate_from(idx);
        }
    }

    /// Get the output of the pipeline (evaluates all dirty stages).
    pub fn output(&mut self) -> &PolyData {
        self.update();
        if let Some(last) = self.stages.last() {
            last.cache.as_ref().unwrap()
        } else {
            &self.input
        }
    }

    /// Get the output of a specific stage by name.
    pub fn stage_output(&mut self, name: &str) -> Option<&PolyData> {
        // Find the stage index
        let idx = self.stages.iter().position(|s| s.name == name)?;
        // Update up to and including this stage
        self.update_to(idx);
        self.stages[idx].cache.as_ref()
    }

    /// Number of stages.
    pub fn num_stages(&self) -> usize {
        self.stages.len()
    }

    /// Check if any stage needs updating.
    pub fn is_dirty(&self) -> bool {
        self.input_dirty || self.stages.iter().any(|s| s.dirty)
    }

    /// Update all dirty stages.
    fn update(&mut self) {
        if !self.stages.is_empty() {
            let last = self.stages.len() - 1;
            self.update_to(last);
        }
    }

    /// Update stages up to and including `to_idx`.
    fn update_to(&mut self, to_idx: usize) {
        for i in 0..=to_idx.min(self.stages.len() - 1) {
            if self.stages[i].dirty || (i == 0 && self.input_dirty) {
                let input = if i == 0 {
                    &self.input
                } else {
                    self.stages[i - 1].cache.as_ref().unwrap()
                };
                let output = (self.stages[i].filter)(input);
                self.stages[i].cache = Some(output);
                self.stages[i].dirty = false;
            }
        }
        self.input_dirty = false;
    }

    /// Builder: add a stage and return self for chaining.
    pub fn then(mut self, name: &str, filter: FilterFn) -> Self {
        self.add(name, filter);
        self
    }

    /// Add a normals computation stage.
    pub fn with_normals(self) -> Self {
        self.then(
            "normals",
            Box::new(|pd| crate::filters::core::normals::compute_normals(pd)),
        )
    }

    /// Add a triangulation stage.
    pub fn with_triangulate(self) -> Self {
        self.then(
            "triangulate",
            Box::new(|pd| crate::filters::core::triangulate::triangulate(pd)),
        )
    }

    /// Add an elevation stage along the Z axis.
    pub fn with_elevation_z(self) -> Self {
        self.then(
            "elevation",
            Box::new(|pd| crate::filters::core::elevation::elevation_z(pd)),
        )
    }

    /// Add a decimation stage.
    pub fn with_decimate(self, target_reduction: f64) -> Self {
        self.then(
            "decimate",
            Box::new(move |pd| crate::filters::core::decimate::decimate(pd, target_reduction)),
        )
    }

    /// Get list of stage names.
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name.as_str()).collect()
    }

    /// Replace the filter function of a named stage (invalidates it and downstream).
    pub fn replace_filter(&mut self, name: &str, filter: FilterFn) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.stages[idx].filter = filter;
            self.invalidate_from(idx);
        }
    }

    fn invalidate_from(&mut self, from_idx: usize) {
        for stage in &mut self.stages[from_idx..] {
            stage.dirty = true;
            stage.cache = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pipeline() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);
        assert_eq!(pipe.output().points.len(), 3);
    }

    #[test]
    fn single_filter() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);
        pipe.add(
            "normals",
            Box::new(|pd| crate::filters::core::normals::compute_normals(pd)),
        );

        let result = pipe.output();
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn caching() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = call_count.clone();
        pipe.add(
            "counter",
            Box::new(move |pd| {
                cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                pd.clone()
            }),
        );

        // First call: evaluates
        pipe.output();
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Second call: cached
        pipe.output();
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Invalidate: re-evaluates
        pipe.invalidate("counter");
        pipe.output();
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn new_input_invalidates_all() {
        let pd1 = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let pd2 = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let mut pipe = Pipeline::new(pd1);
        pipe.add("identity", Box::new(|pd| pd.clone()));

        assert_eq!(pipe.output().points.len(), 3);

        pipe.set_input(pd2);
        assert_eq!(pipe.output().points.len(), 4);
    }

    #[test]
    fn multi_stage() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);
        pipe.add(
            "normals",
            Box::new(|pd| crate::filters::core::normals::compute_normals(pd)),
        );
        pipe.add("copy", Box::new(|pd| pd.clone()));

        assert_eq!(pipe.num_stages(), 2);
        let result = pipe.output();
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn stage_output() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);
        pipe.add(
            "normals",
            Box::new(|pd| crate::filters::core::normals::compute_normals(pd)),
        );
        pipe.add("copy", Box::new(|pd| pd.clone()));

        let normals_out = pipe.stage_output("normals").unwrap();
        assert!(normals_out.point_data().normals().is_some());
    }

    #[test]
    fn builder_chain() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd).with_normals().with_elevation_z();

        assert_eq!(pipe.num_stages(), 2);
        assert_eq!(pipe.stage_names(), vec!["normals", "elevation"]);
        let out = pipe.output();
        assert!(out.point_data().normals().is_some());
    }

    #[test]
    fn replace_filter() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut pipe = Pipeline::new(pd);
        pipe.add("id", Box::new(|pd| pd.clone()));
        pipe.output(); // populate cache

        assert!(!pipe.is_dirty());
        pipe.replace_filter(
            "id",
            Box::new(|pd| crate::filters::core::normals::compute_normals(pd)),
        );
        assert!(pipe.is_dirty());
        let out = pipe.output();
        assert!(out.point_data().normals().is_some());
    }
}
