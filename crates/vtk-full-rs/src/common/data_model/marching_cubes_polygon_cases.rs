/// VTK: `vtkMarchingCubesPolygonCases`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarchingCubesPolygonCases {
    pub edges: [i32; 17],
}

impl MarchingCubesPolygonCases {
    /// VTK: `vtkMarchingCubesPolygonCases::GetCases`.
    pub fn get_cases() -> &'static [Self; 256] {
        &VTK_MARCHING_CUBES_POLYGONCASES
    }
}

const VTK_MARCHING_CUBES_POLYGONCASES: [MarchingCubesPolygonCases; 256] = [
    MarchingCubesPolygonCases {
        edges: [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 1, 0, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 1, 3, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 2, 1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 2, 0, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 3, 8, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 3, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 0, 2, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 3, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 2, 10, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 3, 1, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 1, 11, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 0, 9, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 8, 9, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 0, 3, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 3, 7, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 4, 0, 3, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 0, 9, 11, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 3, 7, 4, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 2, 10, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 3, 2, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 2, 10, 7, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 3, 1, 11, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 0, 1, 11, 10, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 0, 9, 11, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 7, 4, 9, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 1, 0, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 3, 8, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 0, 3, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 0, 4, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 3, 8, 4, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 0, 2, 10, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 0, 4, 5, 3, 3, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 2, 10, 8, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 3, 1, 11, 10, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 1, 11, 10, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 3, 0, 4, 5, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 4, 5, 11, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 7, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 3, 7, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 0, 8, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 1, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 4, 7, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 5, 0, 3, 7, 5, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 0, 8, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 3, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 4, 7, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 2, 10, 7, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 0, 8, 7, 5, 3, 3, 2, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 2, 10, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [4, 3, 1, 11, 10, 4, 7, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 0, 1, 11, 10, 7, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 3, 0, 8, 7, 5, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 7, 5, 11, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 3, 8, 9, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 2, 1, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 1, 5, 6, 3, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 0, 9, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 3, 8, 9, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 0, 2, 10, 8, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 3, 2, 10, 3, 5, 6, 11, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 2, 10, 8, 9, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 1, 5, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 1, 5, 6, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 0, 9, 5, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 5, 6, 10, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 5, 6, 11, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 0, 3, 7, 4, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 5, 6, 11, 3, 7, 4, 8, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 3, 7, 4, 9, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 1, 5, 6, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 1, 5, 6, 4, 0, 3, 7, 4, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 0, 9, 5, 6, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 3, 7, 4, 9, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 3, 5, 6, 11, 3, 7, 4, 8, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 2, 10, 7, 4, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 3, 2, 10, 3, 5, 6, 11, 3, 7, 4, 8, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 2, 10, 7, 4, 9, 3, 5, 6, 11, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 1, 5, 6, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 0, 1, 5, 6, 10, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 0, 9, 5, 6, 10, 3, 7, 4, 8, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 5, 6, 10, 7, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 4, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 4, 4, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 0, 4, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 3, 8, 4, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 1, 9, 4, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 1, 9, 4, 6, 3, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 2, 0, 4, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 3, 8, 4, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 4, 4, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 0, 2, 10, 8, 4, 4, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 0, 4, 6, 11, 3, 3, 2, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 2, 10, 8, 4, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 1, 9, 4, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 0, 1, 9, 4, 6, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 0, 4, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 4, 6, 10, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 7, 6, 11, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 3, 7, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 0, 8, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 3, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 1, 9, 8, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 1, 9, 0, 3, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 0, 8, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 2, 3, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 2, 10, 5, 7, 6, 11, 9, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 0, 2, 10, 7, 6, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 0, 8, 7, 6, 11, 3, 3, 2, 10, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 1, 2, 10, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [7, 3, 1, 9, 8, 7, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 1, 9, 3, 7, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 0, 8, 7, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 7, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 3, 8, 9, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 0, 3, 8, 3, 6, 7, 10, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 0, 9, 11, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 3, 8, 9, 11, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 3, 2, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 2, 6, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 4, 3, 2, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 2, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 1, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 1, 11, 6, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 0, 9, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 6, 7, 8, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 6, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 3, 10, 6, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 4, 6, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 3, 10, 6, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 4, 6, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 5, 0, 3, 10, 6, 4, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 2, 0, 9, 11, 4, 6, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 3, 10, 6, 4, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 2, 6, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 0, 2, 6, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 5, 3, 2, 6, 4, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 2, 6, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 1, 11, 6, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 1, 11, 6, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [7, 3, 0, 9, 11, 6, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 6, 4, 9, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 4, 5, 9, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 3, 4, 5, 9, 3, 6, 7, 10, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 0, 4, 5, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 1, 3, 8, 4, 5, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 4, 5, 9, 3, 6, 7, 10, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 3, 0, 3, 8, 3, 4, 5, 9, 3, 6, 7, 10, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 0, 4, 5, 11, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 3, 8, 4, 5, 11, 3, 6, 7, 10, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 3, 2, 6, 7, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 2, 6, 7, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 0, 4, 5, 4, 3, 2, 6, 7, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 2, 6, 7, 8, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 1, 11, 6, 7, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 1, 11, 6, 7, 8, 3, 4, 5, 9, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 3, 0, 4, 5, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 4, 5, 11, 6, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 6, 5, 9, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 3, 10, 6, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 0, 8, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 3, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 5, 6, 5, 9, 8, 10, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 1, 11, 6, 0, 3, 10, 6, 5, 9, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 0, 8, 10, 6, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 2, 3, 10, 6, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 2, 6, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 2, 6, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 0, 8, 3, 2, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 1, 2, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [7, 3, 1, 11, 6, 5, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 1, 11, 6, 5, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 3, 0, 8, 3, 6, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 6, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 5, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 4, 5, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 4, 5, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [4, 1, 3, 8, 9, 4, 5, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 1, 5, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 2, 1, 5, 7, 10, 3, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 0, 9, 5, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 3, 8, 9, 5, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 2, 11, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 2, 11, 5, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 5, 3, 2, 11, 5, 7, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 2, 11, 5, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 3, 1, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [5, 0, 1, 5, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 0, 9, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 5, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 5, 4, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 0, 3, 10, 11, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 5, 5, 4, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 3, 10, 11, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 1, 5, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 1, 5, 4, 0, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 2, 0, 9, 5, 4, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 2, 3, 10, 3, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 2, 11, 5, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 2, 11, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 0, 9, 6, 3, 2, 11, 5, 4, 8, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 2, 11, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 1, 5, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 0, 1, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 0, 9, 5, 4, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 4, 7, 10, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [3, 0, 3, 8, 5, 4, 7, 10, 11, 9, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            6, 1, 0, 4, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 3, 8, 4, 7, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 1, 9, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 1, 9, 4, 7, 10, 3, 0, 3, 8, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 0, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 3, 8, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 3, 2, 11, 9, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 0, 2, 11, 9, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [7, 1, 0, 4, 7, 3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [3, 1, 2, 11, 3, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [5, 3, 1, 9, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [6, 0, 1, 9, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 3, 0, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 9, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 0, 3, 10, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 1, 0, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 1, 3, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 2, 1, 9, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 2, 1, 9, 0, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 2, 0, 8, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 2, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            5, 3, 2, 11, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 0, 2, 11, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [6, 1, 0, 8, 3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 1, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            4, 3, 1, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            3, 3, 0, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
    MarchingCubesPolygonCases {
        edges: [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ],
    },
];
