# vtk-pure-rs

A pure Rust reimplementation of [VTK 9.6](https://vtk.org/) (The Visualization Toolkit). Translated from the C++ VTK 9.6.0 source ([Kitware/VTK@`00f9418c`](https://github.com/Kitware/VTK/commit/00f9418ca61fa2d3cd75ae78c3978b18fdce12f2)). Not an FFI binding — a ground-up Rust implementation of VTK's core concepts.

* 2026-06-24: A proper audit is taking place

**A few features have not been included; contact if interested in these**


## This is an LLM-mediated faithful (hopefully) translation, not the original code!

Most users should probably first see if the existing original code works for them, unless they have reason otherwise. The original source
may have newer features and it has had more love in terms of fixing bugs. In fact, we aim to replicate bugs if they are present, for the
sake of reproducibility! (but then we might have added a few more in the process)

There are however cases when you might prefer this Rust version. We generally agree with [this page](https://rewrites.bio/)
but more specifically:
* We have had many issues with ensuring that our software works using existing containers (Docker, PodMan, Singularity). One size does not fit all and it eats our resources trying to keep up with every way of delivering software
* Common package managers do not work well. It was great when we had a few Linux distributions with stable procedures, but now there are just too many ecosystems (Homebrew, Conda). Conda has an NP-complete resolver which does not scale. Homebrew is only so-stable. And our dependencies in Python still break. These can no longer be considered professional serious options. Meanwhile, Cargo enables multiple versions of packages to be available, even within the same program(!)
* The future is the web. We deploy software in the web browser, and until now that has meant Javascript. This is a language where even the == operator is broken. Typescript is one step up, but a game changer is the ability to compile Rust code into webassembly, enabling performance and sharing of code with the backend. Translating code to Rust enables new ways of deployment and running code in the browser has especial benefits for science - researchers do not have deep pockets to run servers, so pushing compute to the user enables deployment that otherwise would be impossible
* Old CLI-based utilities are bad for the environment(!). A large amount of compute resources are spent creating and communicating via small files, which we can bypass by using code as libraries. Even better, we can avoid frequent reloading of databases by hoisting this stage, with up to 100x speedups in some cases. Less compute means faster compute and less electricity wasted
* LLM-mediated translations may actually be safer to use than the original code. This article shows that [running the same code on different operating systems can give somewhat different answers](https://doi.org/10.1038/nbt.3820). This is a gap that Rust+Cargo can reduce. Typesafe interfaces also reduce coding mistakes and error handling, as opposed to typical command-line scripting

But:

* **This approach should still be considered experimental**. The LLM technology is immature and has sharp corners. But there are opportunities to reap, and the genie is not going back to the bottle. This translation is as much aimed to learn how to improve the technology and get feedback on the results.
* Translations are not endorsed by the original authors unless otherwise noted. **Do not send bug reports to the original developers**. Use our Github issues page instead.
* **Do not trust the benchmarks on this page**. They are used to help evaluate the translation. If you want improved performance, you generally have to use this code as a library, and use the additional tricks it offers. We generally accept performance losses in order to reduce our dependency issues
* **Check the original Github pages for information about the package**. This README is kept sparse on purpose. It is not meant to be the primary source of information


## Performance vs VTK C++ 9.6

Benchmarked 141 operations against VTK C++ 9.6. **On average 17% faster than C++** (0.83x ratio).

| Category | Count | % |
|---|---|---|
| **Faster than C++** | **109** | **77%** |
| Within 2x | 23 | 16% |
| 2-3x slower | 5 | 4% |
| >3x slower | 4 | 3% |

Biggest wins: signed_distance (32x), poly_data_distance (36x), boolean_union (6x), normals (10x), reflect (11x), shrink_large (10x), connectivity_large (7x), surface_nets (7x).

Uses `Arc<Vec<T>>` copy-on-write storage for zero-copy clone (matching VTK's `ShallowCopy` semantics), `target-cpu=native`, and rayon parallelism for flying edges.

## Test Coverage

- **575 tests** (434 validation + 141 performance)
- **366/396 (92%)** non-extra features tested against VTK C++ reference output
- **30 features** remain untested (exotic I/O, GPU, HyperTreeGrid, Reeb graph internals) — see [TODO.md](TODO.md)

Current manifest-level verification is `cargo check --lib` and `cargo check --examples`. Some checked-in test targets still need monolithic-path cleanup before full `cargo test` is reliable.

## Quick Start

```rust
use vtk_pure_rs::data::*;
use vtk_pure_rs::filters::core::sources::sphere::{sphere, SphereParams};

let mesh = sphere(&SphereParams::default());
println!("points: {}, cells: {}", mesh.points.len(), mesh.polys.num_cells());
```

```toml
[dependencies]
vtk-pure-rs = "0.2"
```

## Build Times

| What you need | Features | Clean release build |
|---------------|----------|---------------------|
| Core filters + common I/O | default | **16s** |
| + all non-extra filter groups + all I/O | + `filters-smooth`, `filters-transform`, etc. + `io-all` | **25s** |
| + extra sources + image + mesh (everything minus GPU) | all non-GPU features | **1m 50s** |

The heavy filter modules (`filters-image` 3000+ modules, `filters-mesh` 800+ modules) are feature-gated so they don't compile unless you need them.

## Feature Counts

| Category | Non-extra | Extra | Total |
|----------|-----------|-------|-------|
| Sources | 64 | 350 | 414 |
| Core Filters | 26 | — | 26 |
| Geometry | 71 | — | 71 |
| Extract | 20 | — | 20 |
| Filter Data | 31 | — | 31 |
| Points | 24 | — | 24 |
| Grid | 19 | — | 19 |
| Transform | 17 | — | 17 |
| Cell | 16 | — | 16 |
| Clip | 11 | — | 11 |
| Statistics | 11 | — | 11 |
| Flow | 11 | — | 11 |
| Distance | 11 | — | 11 |
| Smooth | 10 | — | 10 |
| Subdivide | 8 | — | 8 |
| Normals | 8 | — | 8 |
| Texture | 7 | — | 7 |
| GPU | 5 | — | 5 |
| Boolean | 4 | — | 4 |
| Image | — | 3015 | 3015 |
| Mesh | — | 816 | 816 |
| Core Image | — | 187 | 187 |
| Core Mesh | — | 420 | 420 |
| I/O Formats | 22 | — | 22 |
| **Total** | **396** | **4788** | **5184** |

See [FEATURES.md](FEATURES.md) for the full annotated feature list.

## Module Structure

```
vtk_pure_rs::types      Scalar, ScalarType, CellType, VtkError, BoundingBox, math, color
vtk_pure_rs::data       PolyData, ImageData, UnstructuredGrid, DataArray, CellArray, KdTree, ...
vtk_pure_rs::filters    Sources (64+350), processing (4000+), pipeline, convert, topology
vtk_pure_rs::io         22 formats: VTK, STL, OBJ, PLY, XML, glTF, OFF, DXF, GeoJSON, CSV, ...
vtk_pure_rs::render     Camera, Scene, Actor, Material, Light, ColorMap (15 presets), ...
vtk_pure_rs::render_wgpu  wgpu GPU backend: MSAA, PBR, shadows, SSAO, bloom, volume rendering
```

## Data Structures

| Type | Description |
|------|-------------|
| `PolyData` | Polygonal mesh (triangles, quads, lines, vertices) |
| `ImageData` | Regular grid with implicit coordinates |
| `UnstructuredGrid` | Mixed-cell mesh with explicit connectivity |
| `RectilinearGrid` | Axis-aligned grid with non-uniform spacing |
| `StructuredGrid` | Curvilinear grid with explicit points |
| `DataArray<T>` | N-component typed array with `Arc<Vec<T>>` copy-on-write storage |
| `AnyDataArray` | Type-erased enum over all `DataArray<T>` variants |
| `Table` | Columnar data for analysis |
| `CellGrid` | Discontinuous Galerkin high-order cells |
| `MultiBlockDataSet` | Composite dataset |

## Geometry Sources (64 base + 350 extra)

sphere, cube, cone, cylinder, plane, arrow, disk, line, torus, helix, ellipsoid, capsule, geodesic_sphere, icosphere, superquadric, platonic_solid, frustum, spring, grid, text_3d, wavelet, mobius, klein_bottle, trefoil_knot, boy_surface, seashell, terrain, gear, star, and 30+ more.

Extra sources (behind `sources-extra` feature): airplane, amphora, castle_tower, dna_helix, lighthouse, rocket, space_station, wind_turbine, and 340+ more architectural/scientific/artistic models.

## I/O Formats (22)

| Format | Extension | Read | Write |
|--------|-----------|:----:|:-----:|
| VTK Legacy | `.vtk` | yes | yes |
| VTK XML | `.vtp/.vtu/.vti/.vtr/.vts/.vtm` | yes | yes |
| STL | `.stl` | yes | yes |
| OBJ | `.obj` | yes | yes |
| PLY | `.ply` | yes | yes |
| glTF | `.glb` | yes | yes |
| OFF | `.off` | yes | yes |
| DXF | `.dxf` | yes | yes |
| GeoJSON | `.geojson` | yes | yes |
| CSV/TSV | `.csv/.tsv` | yes | yes |
| EnSight | `.case` | yes | yes |
| FITS | `.fits` | yes | yes |
| LAS | `.las` | yes | yes |
| SEG-Y | `.sgy` | yes | |
| Tecplot | `.dat` | yes | yes |
| BYU | `.byu` | yes | yes |
| Facet | `.facet` | yes | yes |
| XDMF | `.xdmf` | | yes |
| DICOM | `.dcm` | yes | |
| CityGML | `.gml` | yes | |
| Video | `.mp4` etc | | yes* |

*Some checked-in modules reference optional backends such as FFmpeg, HDF5, and GDAL internally, but those backend Cargo features and dependencies are not currently exposed by this manifest.

## Rendering

wgpu-based GPU rendering (behind `render-wgpu` feature):

- Blinn-Phong and Cook-Torrance PBR shading
- Shadow mapping with 3x3 PCF
- Screen-space ambient occlusion (SSAO)
- Bloom post-processing
- Depth-of-field
- GPU volume rendering (ray marching)
- Stereo rendering (side-by-side, anaglyph, top/bottom)
- Multi-viewport split-screen
- Silhouette edges, wireframe, point rendering
- 15 color map presets (jet, viridis, plasma, inferno, etc.)
- Scalar bar, axes widget, axes cube, annotations
- GPU color-ID picking
- LOD, instanced glyphs, clip planes (6 max)
- Fog (linear, exponential)
- Offscreen rendering, PPM/BMP/TGA screenshot export
- CPU ray tracer and Monte Carlo path tracer
- TrueType font rendering (feature-gated `truetype`)

## Examples

```bash
cargo run --example triangle        # basic PolyData construction
cargo run --example shapes          # sphere, cube, cone, cylinder, arrow
cargo run --example isosurface      # marching cubes on a scalar field
cargo run --example scalar_colors   # elevation scalars + color mapping
cargo run --example pipeline_demo   # explicit source/filter chain + I/O
cargo run --example mesh_info       # inspect mesh files
cargo run --features render-wgpu --example showcase        # GPU render showcase
cargo run --features render-wgpu --example volume          # GPU volume rendering
cargo run --features render-wgpu --example headless_render # offscreen rendering
```

## Design Principles

- **Copy-on-write storage** — `Arc<Vec<T>>` in DataArray/CellArray gives zero-copy clone with automatic CoW on mutation, matching VTK's `ShallowCopy` semantics
- **Enum-based type erasure** — `AnyDataArray` is an enum, not `Box<dyn Trait>`
- **Traits over inheritance** — `DataObject` and `DataSet` traits replace class hierarchies
- **Filters as functions** — plain `fn(&PolyData) -> PolyData`, composable without a pipeline
- **Pipeline optional** — `Pipeline` struct available for lazy evaluation + caching when needed
- **Feature-gated heavy modules** — image/mesh filters, wgpu rendering, parallel helpers, and fontdue-backed TrueType support are optional
- **Native CPU targeting** — `.cargo/config.toml` sets `target-cpu=native` for optimal SIMD

## Feature Flags

```toml
[dependencies]
vtk-pure-rs = "0.2"                                          # core filters only
vtk-pure-rs = { version = "0.2", features = ["filters-all"] } # all filters (excl. GPU)
vtk-pure-rs = { version = "0.2", features = ["full"] }        # everything

# Individual filter groups:
vtk-pure-rs = { version = "0.2", features = [
    "filters-smooth",      # smoothing filters
    "filters-transform",   # transform/warp/mirror/extrude
    "filters-subdivide",   # subdivision surfaces
    "filters-cell",        # cell operations
    "filters-boolean",     # boolean mesh operations
    "filters-distance",    # distance/collision/hausdorff
    "filters-image",       # 3000+ image processing filters [heavy]
    "filters-mesh",        # 800+ mesh processing filters [heavy]
    "sources-extra",       # 350 extra geometry sources
    "io-all",              # all currently exposed I/O modules
    "render-wgpu",         # wgpu GPU rendering
    "parallel",            # local parallel/decomposition helpers
    "truetype",            # TrueType font rendering
] }
```
