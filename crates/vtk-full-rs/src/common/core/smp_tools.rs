use super::vtk_type::VtkIdType;
use std::sync::{Mutex, OnceLock};

const SEQUENTIAL_BACKEND: &str = "Sequential";

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmpToolsState {
    desired_number_of_threads: i32,
    nested_parallelism: bool,
}

impl Default for SmpToolsState {
    fn default() -> Self {
        Self {
            desired_number_of_threads: 0,
            nested_parallelism: true,
        }
    }
}

fn state() -> &'static Mutex<SmpToolsState> {
    static STATE: OnceLock<Mutex<SmpToolsState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(SmpToolsState::default()))
}

fn with_state<R>(f: impl FnOnce(&mut SmpToolsState) -> R) -> R {
    let mut guard = state().lock().expect("SMPTools state mutex poisoned");
    f(&mut guard)
}

/// VTK: `vtkSMPTools::Config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmpToolsConfig {
    pub max_number_of_threads: i32,
    pub backend: String,
    pub nested_parallelism: bool,
}

impl Default for SmpToolsConfig {
    fn default() -> Self {
        Self {
            max_number_of_threads: 0,
            backend: SmpTools::get_backend().to_string(),
            nested_parallelism: false,
        }
    }
}

impl SmpToolsConfig {
    /// VTK: `vtkSMPTools::Config(int maxNumberOfThreads, std::string backend, bool nestedParallelism)`.
    pub fn new(
        max_number_of_threads: i32,
        backend: impl Into<String>,
        nested_parallelism: bool,
    ) -> Self {
        Self {
            max_number_of_threads,
            backend: backend.into(),
            nested_parallelism,
        }
    }

    /// VTK: `vtkSMPTools::Config(int maxNumberOfThreads)`.
    pub fn with_max_number_of_threads(max_number_of_threads: i32) -> Self {
        Self {
            max_number_of_threads,
            ..Self::default()
        }
    }

    /// VTK: `vtkSMPTools::Config(std::string backend)`.
    pub fn with_backend(backend: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            ..Self::default()
        }
    }

    /// VTK: `vtkSMPTools::Config(bool nestedParallelism)`.
    pub fn with_nested_parallelism(nested_parallelism: bool) -> Self {
        Self {
            nested_parallelism,
            ..Self::default()
        }
    }
}

/// VTK: `vtkSMPTools`.
pub struct SmpTools;

impl SmpTools {
    /// VTK: `vtkSMPTools::THRESHOLD`.
    pub const THRESHOLD: VtkIdType = 100_000;

    /// VTK: `vtkSMPTools::GetBackend`.
    pub fn get_backend() -> &'static str {
        SEQUENTIAL_BACKEND
    }

    /// VTK: `vtkSMPTools::SetBackend`.
    pub fn set_backend(backend: &str) -> bool {
        backend.eq_ignore_ascii_case(SEQUENTIAL_BACKEND)
    }

    /// VTK: `vtkSMPTools::Initialize`.
    pub fn initialize(num_threads: i32) {
        with_state(|state| {
            state.desired_number_of_threads = num_threads;
        });
    }

    /// VTK: `vtkSMPTools::GetEstimatedNumberOfThreads`.
    pub fn get_estimated_number_of_threads() -> i32 {
        1
    }

    /// VTK: `vtkSMPTools::GetEstimatedDefaultNumberOfThreads`.
    pub fn get_estimated_default_number_of_threads() -> i32 {
        1
    }

    /// VTK: `vtkSMPTools::SetNestedParallelism`.
    pub fn set_nested_parallelism(is_nested: bool) {
        with_state(|state| {
            state.nested_parallelism = is_nested;
        });
    }

    /// VTK: `vtkSMPTools::GetNestedParallelism`.
    pub fn get_nested_parallelism() -> bool {
        with_state(|state| state.nested_parallelism)
    }

    /// VTK: `vtkSMPTools::IsParallelScope`.
    pub fn is_parallel_scope() -> bool {
        false
    }

    /// VTK: `vtkSMPTools::GetSingleThread`.
    pub fn get_single_thread() -> bool {
        true
    }

    /// VTK: `vtkSMPTools::For(first, last, grain, functor)`.
    pub fn r#for(
        first: VtkIdType,
        last: VtkIdType,
        grain: VtkIdType,
        mut f: impl FnMut(VtkIdType, VtkIdType),
    ) {
        let n = last - first;
        if n == 0 {
            return;
        }

        if grain <= 0 || grain >= n {
            f(first, last);
            return;
        }

        let mut begin = first;
        while begin < last {
            let end = (begin + grain).min(last);
            f(begin, end);
            begin = end;
        }
    }
}
