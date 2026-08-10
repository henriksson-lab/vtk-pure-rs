/// VTK: `vtkMarchingSquaresLineCases`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarchingSquaresLineCases {
    pub edges: [i32; 5],
}

impl MarchingSquaresLineCases {
    /// VTK: `vtkMarchingSquaresLineCases::GetCases`.
    pub fn get_cases() -> &'static [Self; 16] {
        &VTK_MARCHING_SQUARES_LINECASES
    }
}

const VTK_MARCHING_SQUARES_LINECASES: [MarchingSquaresLineCases; 16] = [
    MarchingSquaresLineCases {
        edges: [-1, -1, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [0, 3, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [1, 0, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [1, 3, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [2, 1, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [0, 3, 2, 1, -1],
    },
    MarchingSquaresLineCases {
        edges: [2, 0, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [2, 3, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [3, 2, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [0, 2, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [1, 0, 3, 2, -1],
    },
    MarchingSquaresLineCases {
        edges: [1, 2, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [3, 1, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [0, 1, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [3, 0, -1, -1, -1],
    },
    MarchingSquaresLineCases {
        edges: [-1, -1, -1, -1, -1],
    },
];
