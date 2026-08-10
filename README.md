# vtk-pure-rs

A pure Rust reimplementation of [VTK 9.6](https://vtk.org/) (The Visualization Toolkit). Translated from the C++ VTK 9.6.0 source ([Kitware/VTK@`00f9418c`](https://github.com/Kitware/VTK/commit/00f9418ca61fa2d3cd75ae78c3978b18fdce12f2)). Not an FFI binding — a ground-up Rust implementation of VTK's core concepts.

* 2026-07-05: Proper benchmarking started. Regressions being fixed. This will likely take a week
* 2027-07-04: Each file has passed two runs of audit without complaints
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

Original benchmark baseline: VTK C++ comparisons use the vendored Kitware/VTK
source at commit `00f9418ca61f`.

Tracked 141 benchmark operations against VTK C++ 9.6; 141 completed in the latest captured run on 2026-07-14. Average speed ratio: **3.50x** (Rust time / VTK time; lower is better).

The serial rerun used `--test-threads=1`; the Rust test process exited with status 101 because 32 performance-threshold assertions failed after their benchmark rows had been emitted. The per-operation source rows are tracked in `benchmarks/vtk-rs.tsv` in the presentation repository.

RSS is recorded from `/proc/self/status` where available. Average RSS ratio over 141 comparable operations: **0.13x** (Rust RSS / VTK RSS; lower is better).

| Category | Count | % |
|---|---:|---:|
| Faster than C++ | 22 | 16% |
| Within 2x | 21 | 15% |
| 2-3x slower | 40 | 28% |
| >3x slower | 58 | 41% |
| Not captured in latest run | 0 | n/a |

| Algorithm | Parity | Rust ms | VTK ms | Speed ratio | Rust RSS | VTK RSS | RSS ratio | Rust peak delta |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| append_10_small | covered-by-vtk_validation | 0.572 | 0.259 | 2.20x | 7.5 MiB | 178.3 MiB | 0.04x | 1.6 MiB |
| append_3 | covered-by-vtk_validation | 1.427 | 0.058 | 24.71x | 7.8 MiB | 178.5 MiB | 0.04x | 320 KiB |
| append_5_large | covered-by-vtk_validation | 18.435 | 0.600 | 30.74x | 7.8 MiB | 184.9 MiB | 0.04x | 10.3 MiB |
| arrow | covered-by-vtk_validation | 0.087 | 0.348 | 0.25x | 8.4 MiB | 177.5 MiB | 0.05x | 0 KiB |
| boolean_union | covered-by-vtk_validation | 76.807 | 276.501 | 0.28x | 9.4 MiB | 188.3 MiB | 0.05x | 0 KiB |
| butterfly_1 | covered-by-vtk_validation | 22.469 | 4.139 | 5.43x | 11.3 MiB | 180.3 MiB | 0.06x | 0 KiB |
| byu_roundtrip | covered-by-vtk_validation | 6.533 | 3.162 | 2.07x | 11.3 MiB | 179.2 MiB | 0.06x | 0 KiB |
| calculator | covered-by-vtk_validation | 0.169 | 0.734 | 0.23x | 11.3 MiB | 181.1 MiB | 0.06x | 0 KiB |
| catmull_clark_1 | covered-by-vtk_validation | 14.726 | 5.418 | 2.72x | 11.3 MiB | 180.3 MiB | 0.06x | 0 KiB |
| cell_centers | covered-by-vtk_validation | 0.556 | 0.224 | 2.48x | 11.3 MiB | 178.6 MiB | 0.06x | 0 KiB |
| cell_centers_large | covered-by-vtk_validation | 6.379 | 2.867 | 2.22x | 11.3 MiB | 181.7 MiB | 0.06x | 0 KiB |
| cell_quality | covered-by-vtk_validation | 0.361 | 0.442 | 0.82x | 11.3 MiB | 179.1 MiB | 0.06x | 0 KiB |
| cell_quality_large | covered-by-vtk_validation | 2.795 | 4.269 | 0.65x | 11.3 MiB | 180.8 MiB | 0.06x | 0 KiB |
| cell_size | covered-by-vtk_validation | 0.666 | 0.226 | 2.95x | 11.3 MiB | 179.4 MiB | 0.06x | 0 KiB |
| cell_size_large | covered-by-vtk_validation | 7.492 | 2.499 | 3.00x | 11.6 MiB | 181.6 MiB | 0.06x | 0 KiB |
| cell_to_point_data | covered-by-vtk_validation | 1.045 | 0.225 | 4.64x | 11.6 MiB | 182.2 MiB | 0.06x | 0 KiB |
| center_of_mass | covered-by-vtk_validation | 0.038 | 0.052 | 0.74x | 11.6 MiB | 183.7 MiB | 0.06x | 0 KiB |
| clean | covered-by-vtk_validation | 2.095 | 0.738 | 2.84x | 11.6 MiB | 180.9 MiB | 0.06x | 0 KiB |
| clean_3x_large | covered-by-vtk_validation | 58.586 | 16.365 | 3.58x | 13.1 MiB | 186.6 MiB | 0.07x | 2.5 MiB |
| clean_large | covered-by-vtk_validation | 38.000 | 11.838 | 3.21x | 9.0 MiB | 185.0 MiB | 0.05x | 0 KiB |
| clip | covered-by-vtk_validation | 1.378 | 0.654 | 2.11x | 9.0 MiB | 179.6 MiB | 0.05x | 0 KiB |
| clip_closed | covered-by-vtk_validation | 1.791 | 0.477 | 3.76x | 9.0 MiB | 181.8 MiB | 0.05x | 0 KiB |
| clip_large | covered-by-vtk_validation | 19.092 | 7.313 | 2.61x | 12.1 MiB | 182.3 MiB | 0.07x | 0 KiB |
| collision | covered-by-vtk_validation | 5.378 | 3.983 | 1.35x | 12.1 MiB | 180.6 MiB | 0.07x | 0 KiB |
| cone_32 | covered-by-vtk_validation | 0.042 | 0.049 | 0.87x | 12.1 MiB | 175.9 MiB | 0.07x | 0 KiB |
| connectivity | covered-by-vtk_validation | 0.345 | 0.142 | 2.44x | 12.1 MiB | 179.7 MiB | 0.07x | 0 KiB |
| connectivity_large | covered-by-vtk_validation | 33.416 | 17.702 | 1.89x | 15.2 MiB | 192.0 MiB | 0.08x | 0 KiB |
| contour_32 | covered-by-vtk_validation | 5.555 | 0.712 | 7.80x | 15.8 MiB | 179.0 MiB | 0.09x | 0 KiB |
| cube | covered-by-vtk_validation | 0.011 | 0.051 | 0.21x | 15.8 MiB | 176.2 MiB | 0.09x | 0 KiB |
| curvatures_mean | covered-by-vtk_validation | 4.394 | 0.811 | 5.42x | 15.8 MiB | 179.1 MiB | 0.09x | 0 KiB |
| curvatures_gaussian | covered-by-vtk_validation | 0.828 | 0.315 | 2.63x | 15.8 MiB | 179.1 MiB | 0.09x | 0 KiB |
| curvatures_large | covered-by-vtk_validation | 92.929 | 12.452 | 7.46x | 16.5 MiB | 181.1 MiB | 0.09x | 0 KiB |
| cylinder_32 | covered-by-vtk_validation | 0.158 | 0.058 | 2.72x | 16.5 MiB | 176.5 MiB | 0.09x | 0 KiB |
| decimate_50 | covered-by-vtk_validation | 9.803 | 3.556 | 2.76x | 16.5 MiB | 180.0 MiB | 0.09x | 0 KiB |
| decimate_75 | covered-by-vtk_validation | 14.248 | 5.073 | 2.81x | 16.5 MiB | 180.0 MiB | 0.09x | 0 KiB |
| decimate_90 | covered-by-vtk_validation | 16.636 | 6.476 | 2.57x | 16.5 MiB | 180.0 MiB | 0.09x | 0 KiB |
| decimate_50_large | covered-by-vtk_validation | 173.477 | 62.838 | 2.76x | 22.0 MiB | 184.7 MiB | 0.12x | 2.1 MiB |
| delaunay_1000 | covered-by-vtk_validation | 161.010 | 40.410 | 3.98x | 10.2 MiB | 179.4 MiB | 0.06x | 0 KiB |
| delaunay_500 | covered-by-vtk_validation | 41.330 | 15.781 | 2.62x | 10.2 MiB | 179.4 MiB | 0.06x | 0 KiB |
| densify | covered-by-vtk_validation | 2.247 | 0.582 | 3.86x | 10.2 MiB | 179.2 MiB | 0.06x | 0 KiB |
| depth_sort | covered-by-vtk_validation | 0.731 | 0.251 | 2.91x | 10.2 MiB | 178.5 MiB | 0.06x | 0 KiB |
| depth_sort_large | covered-by-vtk_validation | 14.003 | 2.856 | 4.90x | 13.4 MiB | 181.0 MiB | 0.07x | 0 KiB |
| dihedral_angles | covered-by-vtk_validation | 1.738 | 0.465 | 3.74x | 13.4 MiB | 178.0 MiB | 0.08x | 0 KiB |
| disk_32 | covered-by-vtk_validation | 0.242 | 0.079 | 3.07x | 13.4 MiB | 177.8 MiB | 0.08x | 0 KiB |
| distance_to_origin | covered-by-vtk_validation | 0.097 | 0.792 | 0.12x | 13.4 MiB | 177.9 MiB | 0.08x | 0 KiB |
| elevation | covered-by-vtk_validation | 0.089 | 0.036 | 2.51x | 13.4 MiB | 178.1 MiB | 0.07x | 0 KiB |
| elevation_large | covered-by-vtk_validation | 0.477 | 0.234 | 2.04x | 13.4 MiB | 178.6 MiB | 0.07x | 0 KiB |
| extract_cells_half | covered-by-vtk_validation | 0.695 | 0.338 | 2.06x | 13.4 MiB | 185.9 MiB | 0.07x | 0 KiB |
| extract_edges | covered-by-vtk_validation | 2.596 | 1.155 | 2.25x | 13.4 MiB | 180.2 MiB | 0.07x | 0 KiB |
| extract_edges_large | covered-by-vtk_validation | 44.673 | 20.569 | 2.17x | 15.2 MiB | 185.5 MiB | 0.08x | 0 KiB |
| extract_largest | covered-by-vtk_validation | 1.478 | 1.114 | 1.33x | 15.2 MiB | 179.6 MiB | 0.08x | 0 KiB |
| extrude | covered-by-vtk_validation | 0.711 | 0.147 | 4.84x | 15.2 MiB | 178.1 MiB | 0.09x | 0 KiB |
| fe_128 | covered-by-vtk_validation | 73.913 | 5.725 | 12.91x | 28.4 MiB | 181.3 MiB | 0.16x | 9.4 MiB |
| fe_64 | covered-by-vtk_validation | 19.479 | 1.090 | 17.88x | 28.4 MiB | 180.2 MiB | 0.16x | 0 KiB |
| feature_edges_boundary | covered-by-vtk_validation | 0.911 | 0.652 | 1.40x | 28.4 MiB | 179.6 MiB | 0.16x | 0 KiB |
| feature_edges_large | covered-by-vtk_validation | 10.302 | 8.723 | 1.18x | 28.4 MiB | 182.0 MiB | 0.16x | 0 KiB |
| fill_holes | covered-by-vtk_validation | 1.879 | 0.532 | 3.53x | 28.4 MiB | 181.3 MiB | 0.16x | 0 KiB |
| fill_holes_large | covered-by-vtk_validation | 4.879 | 3.770 | 1.29x | 28.4 MiB | 185.3 MiB | 0.15x | 0 KiB |
| glyph_10 | covered-by-vtk_validation | 0.306 | 0.115 | 2.65x | 28.4 MiB | 178.7 MiB | 0.16x | 0 KiB |
| glyph_100 | covered-by-vtk_validation | 0.502 | 0.654 | 0.77x | 28.4 MiB | 179.4 MiB | 0.16x | 0 KiB |
| glyph_50 | covered-by-vtk_validation | 0.667 | 0.505 | 1.32x | 28.4 MiB | 192.0 MiB | 0.15x | 0 KiB |
| gradient | covered-by-vtk_validation | 2.684 | 1.624 | 1.65x | 28.4 MiB | 185.6 MiB | 0.15x | 0 KiB |
| hausdorff | covered-by-vtk_validation | 5.315 | 2.018 | 2.63x | 28.4 MiB | 179.3 MiB | 0.16x | 0 KiB |
| hausdorff_large | covered-by-vtk_validation | 150.803 | 61.206 | 2.46x | 28.4 MiB | 186.0 MiB | 0.15x | 0 KiB |
| hedgehog | covered-by-vtk_validation | 0.474 | 0.081 | 5.83x | 28.4 MiB | 180.1 MiB | 0.16x | 0 KiB |
| hull_200 | covered-by-vtk_validation | 1.848 | 0.479 | 3.86x | 28.4 MiB | 180.7 MiB | 0.16x | 0 KiB |
| linear_subdiv_1 | covered-by-vtk_validation | 5.578 | 1.626 | 3.43x | 28.4 MiB | 179.7 MiB | 0.16x | 0 KiB |
| mask_points_3 | covered-by-vtk_validation | 0.108 | 0.092 | 1.17x | 28.4 MiB | 178.1 MiB | 0.16x | 0 KiB |
| mass_properties | covered-by-vtk_validation | 0.655 | 0.145 | 4.52x | 28.4 MiB | 178.8 MiB | 0.16x | 0 KiB |
| mass_properties_large | covered-by-vtk_validation | 6.278 | 2.334 | 2.69x | 28.4 MiB | 180.2 MiB | 0.16x | 0 KiB |
| mc_128 | covered-by-vtk_validation | 76.897 | 32.665 | 2.35x | 28.7 MiB | 181.7 MiB | 0.16x | 0 KiB |
| mc_64 | covered-by-vtk_validation | 21.414 | 7.352 | 2.91x | 28.7 MiB | 179.8 MiB | 0.16x | 0 KiB |
| mirror | covered-by-vtk_validation | 0.859 | 0.922 | 0.93x | 28.7 MiB | 181.0 MiB | 0.16x | 0 KiB |
| mirror_large | covered-by-vtk_validation | 14.407 | 13.919 | 1.04x | 28.7 MiB | 183.7 MiB | 0.16x | 0 KiB |
| normals | covered-by-vtk_validation | 0.846 | 1.099 | 0.77x | 28.7 MiB | 179.4 MiB | 0.16x | 0 KiB |
| normals_large | covered-by-vtk_validation | 13.663 | 14.210 | 0.96x | 28.7 MiB | 181.2 MiB | 0.16x | 0 KiB |
| obj_large | covered-by-vtk_validation | 422.370 | 59.241 | 7.13x | 28.7 MiB | 181.4 MiB | 0.16x | 0 KiB |
| offset_surface | covered-by-vtk_validation | 0.278 | 0.091 | 3.05x | 28.7 MiB | 186.2 MiB | 0.15x | 0 KiB |
| orient_normals | covered-by-vtk_validation | 4.154 | 1.439 | 2.89x | 28.7 MiB | 183.7 MiB | 0.16x | 0 KiB |
| outline | covered-by-vtk_validation | 0.177 | 0.076 | 2.33x | 28.7 MiB | 178.8 MiB | 0.16x | 0 KiB |
| pipeline_normals_smooth | covered-by-vtk_validation | 96.971 | 32.282 | 3.00x | 28.7 MiB | 180.3 MiB | 0.16x | 0 KiB |
| plane_32 | covered-by-vtk_validation | 0.413 | 0.100 | 4.14x | 28.7 MiB | 177.8 MiB | 0.16x | 0 KiB |
| ply_roundtrip | covered-by-vtk_validation | 2.385 | 1.387 | 1.72x | 28.7 MiB | 179.1 MiB | 0.16x | 0 KiB |
| ply_large | covered-by-vtk_validation | 36.536 | 17.287 | 2.11x | 28.7 MiB | 182.5 MiB | 0.16x | 0 KiB |
| point_density | covered-by-vtk_validation | 5.582 | 1.000 | 5.58x | 28.7 MiB | 177.6 MiB | 0.16x | 0 KiB |
| point_to_cell_data | covered-by-vtk_validation | 1.041 | 0.214 | 4.87x | 28.7 MiB | 182.2 MiB | 0.16x | 0 KiB |
| poly_data_distance | covered-by-vtk_validation | 12.377 | 30.459 | 0.41x | 28.7 MiB | 182.3 MiB | 0.16x | 0 KiB |
| probe_100 | covered-by-vtk_validation | 1.759 | 0.538 | 3.27x | 28.7 MiB | 179.7 MiB | 0.16x | 0 KiB |
| quadric_clustering | covered-by-vtk_validation | 1.158 | 1.075 | 1.08x | 28.7 MiB | 185.0 MiB | 0.16x | 0 KiB |
| quadric_decimate_50 | covered-by-vtk_validation | 10.207 | 3.834 | 2.66x | 28.7 MiB | 180.1 MiB | 0.16x | 0 KiB |
| quadric_decimate_50_large | covered-by-vtk_validation | 181.521 | 74.280 | 2.44x | 28.7 MiB | 192.1 MiB | 0.15x | 0 KiB |
| quadric_clustering_large | covered-by-vtk_validation | 16.732 | 7.199 | 2.32x | 28.7 MiB | 189.7 MiB | 0.15x | 0 KiB |
| reflect | covered-by-vtk_validation | 1.635 | 4.129 | 0.40x | 28.7 MiB | 179.9 MiB | 0.16x | 0 KiB |
| reflect_large | covered-by-vtk_validation | 26.688 | 23.243 | 1.15x | 28.7 MiB | 183.6 MiB | 0.16x | 0 KiB |
| reverse_sense | covered-by-vtk_validation | 0.642 | 0.090 | 7.16x | 28.7 MiB | 178.3 MiB | 0.16x | 0 KiB |
| reverse_sense_large | covered-by-vtk_validation | 7.376 | 1.257 | 5.87x | 28.7 MiB | 181.1 MiB | 0.16x | 0 KiB |
| ribbon | covered-by-vtk_validation | 0.087 | 0.105 | 0.83x | 28.7 MiB | 183.7 MiB | 0.16x | 0 KiB |
| rotation_extrude | covered-by-vtk_validation | 0.087 | 0.215 | 0.40x | 28.7 MiB | 178.1 MiB | 0.16x | 0 KiB |
| ruled_surface | covered-by-vtk_validation | 0.077 | 1.000 | 0.08x | 28.7 MiB | 178.3 MiB | 0.16x | 0 KiB |
| separate_cells | covered-by-vtk_validation | 1.074 | 0.806 | 1.33x | 28.7 MiB | 179.1 MiB | 0.16x | 0 KiB |
| shrink | covered-by-vtk_validation | 1.038 | 0.648 | 1.60x | 28.7 MiB | 178.6 MiB | 0.16x | 0 KiB |
| shrink_large | covered-by-vtk_validation | 17.586 | 10.602 | 1.66x | 28.7 MiB | 180.9 MiB | 0.16x | 0 KiB |
| signed_distance_32 | covered-by-vtk_validation | 98.837 | 188.808 | 0.52x | 28.7 MiB | 180.6 MiB | 0.16x | 0 KiB |
| silhouette | covered-by-vtk_validation | 2.554 | 1.000 | 2.55x | 28.7 MiB | 179.7 MiB | 0.16x | 0 KiB |
| slice | covered-by-vtk_validation | 0.348 | 0.442 | 0.79x | 28.7 MiB | 183.7 MiB | 0.16x | 0 KiB |
| slice_large | covered-by-vtk_validation | 2.187 | 1.149 | 1.90x | 28.7 MiB | 187.2 MiB | 0.15x | 0 KiB |
| smooth_20 | covered-by-vtk_validation | 8.811 | 1.147 | 7.68x | 28.7 MiB | 179.2 MiB | 0.16x | 0 KiB |
| smooth_20_constrained | covered-by-vtk_validation | 8.272 | 1.146 | 7.22x | 28.7 MiB | 181.0 MiB | 0.16x | 0 KiB |
| smooth_20_large | covered-by-vtk_validation | 91.299 | 19.406 | 4.70x | 28.7 MiB | 180.9 MiB | 0.16x | 0 KiB |
| smooth_50 | covered-by-vtk_validation | 15.712 | 1.809 | 8.69x | 28.7 MiB | 179.1 MiB | 0.16x | 0 KiB |
| smooth_50_large | covered-by-vtk_validation | 101.503 | 30.896 | 3.29x | 28.7 MiB | 181.1 MiB | 0.16x | 0 KiB |
| sphere_128x128 | covered-by-vtk_validation | 4.777 | 1.312 | 3.64x | 28.7 MiB | 178.8 MiB | 0.16x | 0 KiB |
| sphere_32x32 | covered-by-vtk_validation | 0.489 | 0.093 | 5.23x | 28.7 MiB | 177.6 MiB | 0.16x | 0 KiB |
| sphere_64x64 | covered-by-vtk_validation | 1.212 | 0.364 | 3.33x | 28.7 MiB | 177.5 MiB | 0.16x | 0 KiB |
| spline | covered-by-vtk_validation | 0.037 | 0.043 | 0.87x | 28.7 MiB | 178.1 MiB | 0.16x | 0 KiB |
| stl_roundtrip | covered-by-vtk_validation | 6.454 | 1.630 | 3.96x | 28.7 MiB | 179.8 MiB | 0.16x | 0 KiB |
| stl_large | covered-by-vtk_validation | 106.352 | 22.582 | 4.71x | 28.7 MiB | 184.0 MiB | 0.16x | 0 KiB |
| subdivide_1 | covered-by-vtk_validation | 18.496 | 3.333 | 5.55x | 28.7 MiB | 181.2 MiB | 0.16x | 0 KiB |
| subdivide_2 | covered-by-vtk_validation | 100.419 | 16.124 | 6.23x | 28.7 MiB | 189.7 MiB | 0.15x | 0 KiB |
| surface_nets_32 | covered-by-vtk_validation | 26.191 | 16.784 | 1.56x | 28.7 MiB | 180.6 MiB | 0.16x | 0 KiB |
| texture_map_sphere | covered-by-vtk_validation | 0.505 | 0.185 | 2.73x | 28.7 MiB | 177.8 MiB | 0.16x | 0 KiB |
| threshold | covered-by-vtk_validation | 0.842 | 0.198 | 4.25x | 28.7 MiB | 179.8 MiB | 0.16x | 0 KiB |
| topology_analysis | covered-by-vtk_validation | 1.927 | 0.722 | 2.67x | 28.7 MiB | 179.9 MiB | 0.16x | 0 KiB |
| topology_analysis_large | covered-by-vtk_validation | 33.032 | 8.446 | 3.91x | 28.7 MiB | 182.0 MiB | 0.16x | 0 KiB |
| transform | covered-by-vtk_validation | 0.315 | 0.103 | 3.05x | 28.7 MiB | 179.7 MiB | 0.16x | 0 KiB |
| transform_large | covered-by-vtk_validation | 1.663 | 0.335 | 4.96x | 28.7 MiB | 189.7 MiB | 0.15x | 0 KiB |
| triangle_strips | covered-by-vtk_validation | 2.057 | 0.350 | 5.88x | 28.7 MiB | 178.5 MiB | 0.16x | 0 KiB |
| triangle_strips_large | covered-by-vtk_validation | 34.585 | 4.400 | 7.86x | 28.7 MiB | 180.9 MiB | 0.16x | 0 KiB |
| triangulate | covered-by-vtk_validation | 0.121 | 0.039 | 3.13x | 28.7 MiB | 177.6 MiB | 0.16x | 0 KiB |
| triangulate_large | covered-by-vtk_validation | 0.592 | 0.058 | 10.16x | 28.7 MiB | 179.5 MiB | 0.16x | 0 KiB |
| tube | covered-by-vtk_validation | 0.317 | 0.057 | 5.53x | 28.7 MiB | 178.4 MiB | 0.16x | 0 KiB |
| tube_large | covered-by-vtk_validation | 0.692 | 0.257 | 2.69x | 28.7 MiB | 192.3 MiB | 0.15x | 0 KiB |
| validate | covered-by-vtk_validation | 1.071 | 0.790 | 1.36x | 28.7 MiB | 177.9 MiB | 0.16x | 0 KiB |
| voronoi_200 | covered-by-vtk_validation | 11.740 | 10.000 | 1.17x | 28.7 MiB | 179.1 MiB | 0.16x | 0 KiB |
| voxel_grid | covered-by-vtk_validation | 1.265 | 5.056 | 0.25x | 28.7 MiB | 177.4 MiB | 0.16x | 0 KiB |
| vtk_roundtrip | covered-by-vtk_validation | 15.070 | 4.127 | 3.65x | 28.7 MiB | 179.7 MiB | 0.16x | 0 KiB |
| vtk_large | covered-by-vtk_validation | 259.665 | 65.872 | 3.94x | 28.7 MiB | 181.9 MiB | 0.16x | 0 KiB |
| vtp_roundtrip | covered-by-vtk_validation | 17.608 | 4.365 | 4.03x | 28.7 MiB | 181.2 MiB | 0.16x | 0 KiB |
| vtp_large | covered-by-vtk_validation | 314.505 | 58.325 | 5.39x | 28.7 MiB | 183.9 MiB | 0.16x | 0 KiB |
| warp_scalar | covered-by-vtk_validation | 0.181 | 0.111 | 1.63x | 28.7 MiB | 179.7 MiB | 0.16x | 0 KiB |
| windowed_sinc_20 | covered-by-vtk_validation | 4.553 | 1.376 | 3.31x | 28.7 MiB | 177.9 MiB | 0.16x | 0 KiB |

Benchmark command:

```bash
cargo test --test vtk_perf --features filters-smooth,filters-transform,filters-subdivide,filters-cell,filters-statistics,filters-texture,filters-flow,filters-boolean,filters-data,filters-distance -- --nocapture --test-threads=1
```

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
