# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

vtk-rs is a pure Rust reimplementation of VTK (The Visualization Toolkit). The `VTK/` directory contains the C++ 9.6.0 source as reference — this is **not** an FFI bindings project.

## Build / Test / Lint

```bash
cargo check --lib                        # fast library check
cargo check --examples                   # check all example targets
cargo build                              # build the package
cargo test --lib                         # run unit tests; currently exposes import errors in some source tests
cargo test --test vtk_validation         # run C++ reference validation tests
cargo test --test io_roundtrip           # run I/O roundtrip tests
cargo clippy --lib                       # lint the library; not currently clean with -D warnings
```

## Workspace Structure

This repository is currently a single Cargo package (`vtk-pure-rs`) with a monolithic module tree under `src/`:

**Core:** `src/types` and `src/data` — foundational types and data model
**Filters:** `src/filters` — core filters plus feature-gated groups such as image, mesh, smooth, transform, cell, statistics, texture, flow, boolean, grid, distance, and GPU
**I/O:** `src/io` — feature-gated by `io-common`, with per-format modules under `src/io/*`
**Rendering:** `src/render` and `src/render_wgpu` — gated by `render` and `render-wgpu`
**Parallel:** `src/parallel` — gated by `parallel`

The public crate is imported as `vtk_pure_rs`; there are no active split crates such as `vtk_data`, `vtk_filters`, or `vtk_render` in this manifest.

### Key vtk-filters modules

**Sources (414):** sphere, cube, cone, cylinder, plane, arrow, disk, line, point_source, regular_polygon, arc, superquadric, platonic_solid, frustum, parametric, bounding_box_source, axes, torus, helix, ellipsoid, spring, capsule, geodesic_sphere, grid, text_3d, wavelet, circle, mobius, star, noise_field, ring, klein_bottle, trefoil_knot, cross, boy_surface, spiral, icosphere, mobius_strip, gear, grid_2d, earth, sector, plus generated extra sources gated by `sources-extra`

**Infrastructure:** pipeline (lazy evaluation + caching), convert (dataset conversions), topology (manifold/euler/boundary analysis), merge (combine meshes), selection_extract (apply selections), io_utils (auto-format read/write)

## Key Design Decisions

- **Copy-on-write data arrays** — `DataArray` and related containers use `Arc<Vec<T>>` storage for cheap clones while keeping mutation explicit through copy-on-write.
- **Enum-based type erasure** — `AnyDataArray` is an enum over `DataArray<f32>`, `DataArray<f64>`, etc. (closed set of scalar types). Prefer this over `Box<dyn Trait>`.
- **Traits over inheritance** — `DataObject` and `DataSet` traits replace VTK's class hierarchy. `DataSet` has default methods: `center()`, `diagonal()`, `is_empty()`.
- **Pipeline system** — `Pipeline` struct with lazy evaluation, caching, and invalidation. Builder API with `with_normals()`, `with_decimate()`, etc.
- **Scalar visualization** — `Actor` supports `Coloring::ScalarMap` with `ColorMap` (15 presets + `by_name()` lookup). Active scalars from point data are mapped through the color map.
- **PBR rendering** — Blinn-Phong and Cook-Torrance PBR (metallic/roughness) selectable per-actor via `material.pbr` flag.
- **wgpu rendering** — 4x MSAA, per-actor model matrix (position/scale), 6 clip planes, flat shading (dpdx/dpdy), backface culling, silhouette edges.
- **VTK legacy format** uses version 4.2 (legacy cell format: `npts id0 id1 ...`). Binary is big-endian.
- **Prelude module** — `use vtk_pure_rs::prelude::*` for quick starts.
- **WASM oriented** — non-GPU modules are intended to support wasm32-unknown-unknown, but verify the selected feature set before relying on it.

## VTK C++ Reference Architecture

The C++ source at `VTK/` is organized as a modular toolkit. Key reference files:

- `VTK/Common/DataModel/vtkPolyData.h` — 4-CellArray structure (verts/lines/polys/strips)
- `VTK/Common/DataModel/vtkCellArray.h` — offsets+connectivity design
- `VTK/IO/Legacy/vtkDataWriter.cxx` — legacy format output details
- `VTK/IO/Legacy/vtkDataReader.cxx` — legacy format parsing
- `VTK/Filters/Sources/vtkSphereSource.h` — geometry source pattern
