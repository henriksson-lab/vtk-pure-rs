# vtk-full-rs

This crate is the VTK-shaped translation target.

The goal is eventual coverage of all extracted VTK symbols. Missing VTK work is
tracked in `../../docs/audit/vtk_coverage.csv`; the crate does not require
placeholder Rust stubs for every missing VTK function.

Code should be copied or translated here only when its origin is represented in
`../../docs/audit/mapping.csv` and the migration action in
`../../docs/audit/new_crate_migration.csv` permits it.

Files should follow the upstream VTK layout where practical:

- `VTK/Common/DataModel/vtkPolyData.cxx` -> `src/common/data_model/poly_data.rs`
- header declarations and implementation bodies are merged into the same Rust
  file,
- original VTK function order is preserved where practical,
- implementation work follows `../../docs/audit/vtk_porting_order.csv` loosely
  bottom-up.

## Storage and Copy Semantics

This crate is a VTK-shaped rewrite. Large mutable payloads should use shared
storage plus copy-on-write mutation so VTK-style shallow copies stay cheap:

- use `Arc<...Storage>` for arrays, points, cell arrays, field data, dataset
  attributes, graph internals, and other large payloads,
- use `Arc::make_mut` inside mutating APIs,
- make `shallow_copy_from` share storage when VTK would share reference-counted
  arrays or internals,
- make `deep_copy_from` clone storage,
- keep small scalar metadata as ordinary fields.
