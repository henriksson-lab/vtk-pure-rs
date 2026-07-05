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

Tracked 141 benchmark operations against VTK C++ 9.6; 141 completed in the latest captured run. Average speed ratio: **10.82x** (Rust time / VTK time; lower is better).

RSS is recorded from `/proc/self/status` where available. Average RSS ratio over 141 comparable operations: **0.94x** (Rust RSS / VTK RSS; lower is better).

| Category | Count | % |
|---|---:|---:|
| Faster than C++ | 9 | 6% |
| Within 2x | 7 | 5% |
| 2-3x slower | 10 | 7% |
| >3x slower | 115 | 82% |
| Not captured in latest run | 0 | n/a |

| Algorithm | Parity | Rust ms | VTK ms | Speed ratio | Rust RSS | VTK RSS | RSS ratio | Rust peak delta |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| append_10_small | reference fixture: filter_append | 0.690 | 0.259 | 2.66x | 53.8 MiB | 178.3 MiB | 0.30x | 46.6 MiB |
| append_3 | reference fixture: filter_append | 3.07 | 0.058 | 53.18x | 98.2 MiB | 178.5 MiB | 0.55x | 91.4 MiB |
| append_5_large | reference fixture: filter_append | 22.55 | 0.600 | 37.59x | 8.1 MiB | 184.9 MiB | 0.04x | 12.8 MiB |
| arrow | reference fixture: source_arrow | 0.168 | 0.348 | 0.48x | 13.8 MiB | 177.5 MiB | 0.08x | 6.9 MiB |
| boolean_union | reference fixture: filter_boolean_union | 173.8 | 276.5 | 0.63x | 274.1 MiB | 188.3 MiB | 1.46x | 271.6 MiB |
| butterfly_1 | reference fixture: filter_butterfly_1 | 58.47 | 4.14 | 14.13x | 250.3 MiB | 180.3 MiB | 1.39x | 242.2 MiB |
| byu_roundtrip | reference fixture: io_byu | 19.04 | 3.16 | 6.02x | 170.3 MiB | 179.2 MiB | 0.95x | 163.9 MiB |
| calculator | reference fixture: filter_calculator | 0.180 | 0.734 | 0.24x | 20.9 MiB | 181.1 MiB | 0.12x | 14.4 MiB |
| catmull_clark_1 | reference fixture: filter_catmull_clark_1 | 33.57 | 5.42 | 6.20x | 218.1 MiB | 180.3 MiB | 1.21x | 210.9 MiB |
| cell_centers | reference fixture: filter_cell_centers | 1.07 | 0.224 | 4.76x | 56.6 MiB | 178.6 MiB | 0.32x | 53.9 MiB |
| cell_centers_large | reference fixture: filter_cell_centers | 15.23 | 2.87 | 5.31x | 168.5 MiB | 181.7 MiB | 0.93x | 135.3 MiB |
| cell_quality | reference fixture: filter_cell_quality | 2.10 | 0.442 | 4.74x | 78.0 MiB | 179.1 MiB | 0.44x | 72.9 MiB |
| cell_quality_large | reference fixture: filter_cell_quality | 33.10 | 4.27 | 7.75x | 215.6 MiB | 180.8 MiB | 1.19x | 181.1 MiB |
| cell_size | reference fixture: filter_cell_size | 1.12 | 0.226 | 4.98x | 54.7 MiB | 179.4 MiB | 0.31x | 54.2 MiB |
| cell_size_large | reference fixture: filter_cell_size | 17.76 | 2.50 | 7.11x | 171.1 MiB | 181.6 MiB | 0.94x | 127.9 MiB |
| cell_to_point_data | reference fixture: filter_cell_to_point_data | 1.97 | 0.225 | 8.75x | 7.5 MiB | 182.2 MiB | 0.04x | 320 KiB |
| center_of_mass | reference fixture: filter_center_of_mass | 0.923 | 0.052 | 17.89x | 53.8 MiB | 183.7 MiB | 0.29x | 53.9 MiB |
| clean | reference fixture: filter_clean | 6.66 | 0.738 | 9.03x | 22.5 MiB | 180.9 MiB | 0.12x | 15.6 MiB |
| clean_3x_large | reference fixture: filter_clean | 190.0 | 16.37 | 11.61x | 8.5 MiB | 186.6 MiB | 0.05x | 26.6 MiB |
| clean_large | reference fixture: filter_clean | 125.7 | 11.84 | 10.62x | 21.3 MiB | 185.0 MiB | 0.12x | 25.7 MiB |
| clip | reference fixture: filter_clip_plane | 9.19 | 0.654 | 14.05x | 134.4 MiB | 179.6 MiB | 0.75x | 123.4 MiB |
| clip_closed | reference fixture: filter_clip_closed | 7.12 | 0.477 | 14.94x | 116.5 MiB | 181.8 MiB | 0.64x | 119.2 MiB |
| clip_large | reference fixture: filter_clip_plane | 32.24 | 7.31 | 4.41x | 9.7 MiB | 182.3 MiB | 0.05x | 2.6 MiB |
| collision | reference fixture: filter_collision | 13.32 | 3.98 | 3.34x | 144.0 MiB | 180.6 MiB | 0.80x | 134.7 MiB |
| cone_32 | reference fixture: source_cone_32 | 0.074 | 0.049 | 1.53x | 18.1 MiB | 175.9 MiB | 0.10x | 6.9 MiB |
| connectivity | reference fixture: filter_connectivity | 0.719 | 0.142 | 5.07x | 58.6 MiB | 179.7 MiB | 0.33x | 48.9 MiB |
| connectivity_large | reference fixture: filter_connectivity_large | 86.30 | 17.70 | 4.87x | 272.5 MiB | 192.0 MiB | 1.42x | 262.8 MiB |
| contour_32 | reference fixture: filter_contour | 10.77 | 0.712 | 15.13x | 7.6 MiB | 179.0 MiB | 0.04x | 1.6 MiB |
| cube | reference fixture: source_cube | 0.064 | 0.051 | 1.25x | 14.0 MiB | 176.2 MiB | 0.08x | 3.8 MiB |
| curvatures_gaussian | reference fixture: filter_curvatures_gaussian | 14.43 | 0.315 | 45.82x | 156.5 MiB | 179.1 MiB | 0.87x | 142.5 MiB |
| curvatures_large | reference fixture: filter_curvatures_large | 258.2 | 12.45 | 20.73x | 263.0 MiB | 181.1 MiB | 1.45x | 239.4 MiB |
| curvatures_mean | reference fixture: filter_curvatures_mean | 14.70 | 0.811 | 18.12x | 152.2 MiB | 179.1 MiB | 0.85x | 137.8 MiB |
| cylinder_32 | reference fixture: source_cylinder_32 | 0.097 | 0.058 | 1.67x | 13.8 MiB | 176.5 MiB | 0.08x | 6.6 MiB |
| decimate_50 | reference fixture: filter_decimate_50 | 37.54 | 3.56 | 10.56x | 217.8 MiB | 180.0 MiB | 1.21x | 212.5 MiB |
| decimate_50_large | reference fixture: filter_decimate_50 | 465.5 | 62.84 | 7.41x | 291.0 MiB | 184.7 MiB | 1.58x | 258.7 MiB |
| decimate_75 | reference fixture: filter_decimate_75 | 55.20 | 5.07 | 10.88x | 249.1 MiB | 180.0 MiB | 1.38x | 232.5 MiB |
| decimate_90 | reference fixture: filter_decimate_90 | 65.11 | 6.48 | 10.05x | 237.2 MiB | 180.0 MiB | 1.32x | 245.7 MiB |
| delaunay_1000 | reference fixture: filter_delaunay_2d | 639.8 | 40.41 | 15.83x | 288.1 MiB | 179.4 MiB | 1.61x | 278.9 MiB |
| delaunay_500 | reference fixture: filter_delaunay_500 | 256.9 | 15.78 | 16.28x | 280.2 MiB | 179.4 MiB | 1.56x | 265.4 MiB |
| densify | reference fixture: filter_densify | 8.40 | 0.582 | 14.43x | 121.2 MiB | 179.2 MiB | 0.68x | 116.3 MiB |
| depth_sort | reference fixture: filter_depth_sort | 2.39 | 0.251 | 9.53x | 98.5 MiB | 178.5 MiB | 0.55x | 70.1 MiB |
| depth_sort_large | reference fixture: filter_depth_sort | 42.73 | 2.86 | 14.96x | 223.4 MiB | 181.0 MiB | 1.23x | 171.1 MiB |
| dihedral_angles | reference fixture: filter_dihedral_angles | 7.94 | 0.465 | 17.09x | 121.5 MiB | 178.0 MiB | 0.68x | 95.1 MiB |
| disk_32 | reference fixture: source_disk | 0.179 | 0.079 | 2.26x | 51.3 MiB | 177.8 MiB | 0.29x | 26.0 MiB |
| distance_to_origin | reference fixture: filter_distance_to_origin | 0.124 | 0.792 | 0.16x | 34.8 MiB | 177.9 MiB | 0.20x | 9.2 MiB |
| elevation | reference fixture: filter_elevation | 0.196 | 0.036 | 5.52x | 52.2 MiB | 178.1 MiB | 0.29x | 11.5 MiB |
| elevation_large | reference fixture: filter_elevation | 2.99 | 0.234 | 12.79x | 116.1 MiB | 178.6 MiB | 0.65x | 55.0 MiB |
| extract_cells_half | reference fixture: filter_extract_cells_half | 4.99 | 0.338 | 14.75x | 112.8 MiB | 185.9 MiB | 0.61x | 73.2 MiB |
| extract_edges | reference fixture: filter_extract_edges | 13.75 | 1.15 | 11.91x | 149.4 MiB | 180.2 MiB | 0.83x | 93.6 MiB |
| extract_edges_large | reference fixture: filter_extract_edges | 230.6 | 20.57 | 11.21x | 280.5 MiB | 185.5 MiB | 1.51x | 215.6 MiB |
| extract_largest | reference fixture: filter_extract_largest | 11.96 | 1.11 | 10.73x | 151.2 MiB | 179.6 MiB | 0.84x | 90.1 MiB |
| extrude | reference fixture: filter_extrude | 1.17 | 0.147 | 7.95x | 92.0 MiB | 178.1 MiB | 0.52x | 31.6 MiB |
| fe_128 | reference fixture: filter_fe_128 | 239.7 | 5.73 | 41.86x | 255.2 MiB | 181.3 MiB | 1.41x | 223.3 MiB |
| fe_64 | reference fixture: filter_fe_64 | 35.00 | 1.09 | 32.12x | 9.6 MiB | 180.2 MiB | 0.05x | 4.1 MiB |
| feature_edges_boundary | reference fixture: filter_feature_edges | 9.19 | 0.652 | 14.11x | 141.5 MiB | 179.6 MiB | 0.79x | 47.4 MiB |
| feature_edges_large | reference fixture: filter_feature_edges | 151.4 | 8.72 | 17.36x | 277.5 MiB | 182.0 MiB | 1.52x | 176.4 MiB |
| fill_holes | reference fixture: filter_fill_holes | 6.09 | 0.532 | 11.44x | 9.7 MiB | 181.3 MiB | 0.05x | 4.1 MiB |
| fill_holes_large | reference fixture: filter_fill_holes | 240.7 | 3.77 | 63.84x | 12.7 MiB | 185.3 MiB | 0.07x | 7.1 MiB |
| glyph_10 | reference fixture: filter_glyph | 1.01 | 0.115 | 8.71x | 120.6 MiB | 178.7 MiB | 0.67x | 5.9 MiB |
| glyph_100 | reference fixture: filter_glyph | 4.78 | 0.654 | 7.31x | 141.5 MiB | 179.4 MiB | 0.79x | 14.3 MiB |
| glyph_50 | reference fixture: filter_glyph | 4.63 | 0.505 | 9.17x | 143.4 MiB | 192.0 MiB | 0.75x | 16.1 MiB |
| gradient | reference fixture: filter_gradient | 10.34 | 1.62 | 6.37x | 169.9 MiB | 185.6 MiB | 0.92x | 42.9 MiB |
| hausdorff | reference fixture: filter_hausdorff | 11.81 | 2.02 | 5.85x | 173.7 MiB | 179.3 MiB | 0.97x | 46.4 MiB |
| hausdorff_large | reference fixture: filter_hausdorff | 306.9 | 61.21 | 5.01x | 281.2 MiB | 186.0 MiB | 1.51x | 157.8 MiB |
| hedgehog | reference fixture: filter_hedgehog | 3.56 | 0.081 | 43.79x | 144.0 MiB | 180.1 MiB | 0.80x | 9.1 MiB |
| hull_200 | reference fixture: filter_hull_200 | 7.18 | 0.479 | 14.99x | 174.3 MiB | 180.7 MiB | 0.96x | 32.8 MiB |
| linear_subdiv_1 | reference fixture: filter_subdivide_1 | 27.26 | 1.63 | 16.76x | 218.1 MiB | 179.7 MiB | 1.21x | 84.7 MiB |
| mask_points_3 | reference fixture: filter_mask_points_3 | 0.236 | 0.092 | 2.58x | 144.0 MiB | 178.1 MiB | 0.81x | 320 KiB |
| mass_properties | reference fixture: filter_mass_properties | 0.861 | 0.145 | 5.95x | 149.0 MiB | 178.8 MiB | 0.83x | 5.0 MiB |
| mass_properties_large | reference fixture: filter_mass_properties | 14.54 | 2.33 | 6.23x | 203.4 MiB | 180.2 MiB | 1.13x | 55.0 MiB |
| mc_128 | reference fixture: filter_mc_128 | 678.7 | 32.66 | 20.78x | 264.1 MiB | 181.7 MiB | 1.45x | 147.0 MiB |
| mc_64 | reference fixture: filter_mc_64 | 78.64 | 7.35 | 10.70x | 6.9 MiB | 179.8 MiB | 0.04x | 6.9 MiB |
| mirror | reference fixture: filter_mirror | 2.02 | 0.922 | 2.19x | 167.4 MiB | 181.0 MiB | 0.92x | 17.9 MiB |
| mirror_large | reference fixture: filter_mirror | 34.07 | 13.92 | 2.45x | 238.8 MiB | 183.7 MiB | 1.30x | 82.2 MiB |
| normals | reference fixture: filter_normals | 1.88 | 1.10 | 1.71x | 169.5 MiB | 179.4 MiB | 0.95x | 17.1 MiB |
| normals_large | reference fixture: filter_normals | 31.45 | 14.21 | 2.21x | 232.2 MiB | 181.2 MiB | 1.28x | 68.7 MiB |
| obj_large | reference fixture: io_obj | 526.5 | 59.24 | 8.89x | 264.9 MiB | 181.4 MiB | 1.46x | 122.8 MiB |
| offset_surface | reference fixture: filter_offset_surface | 1.65 | 0.091 | 18.14x | 170.0 MiB | 186.2 MiB | 0.91x | 2.6 MiB |
| orient_normals | validation test: filter_orient_consistent | 9.71 | 1.44 | 6.74x | 195.0 MiB | 183.7 MiB | 1.06x | 25.7 MiB |
| outline | reference fixture: filter_outline | 0.115 | 0.076 | 1.52x | 170.2 MiB | 178.8 MiB | 0.95x | 640 KiB |
| pipeline_normals_smooth | reference fixture: filter_normals | 315.0 | 32.28 | 9.76x | 282.6 MiB | 180.3 MiB | 1.57x | 114.9 MiB |
| plane_32 | reference fixture: source_plane_32x32 | 0.925 | 0.100 | 9.26x | 171.8 MiB | 177.8 MiB | 0.97x | 1.7 MiB |
| ply_large | reference fixture: io_ply | 133.9 | 17.29 | 7.75x | 278.1 MiB | 182.5 MiB | 1.52x | 107.0 MiB |
| ply_roundtrip | reference fixture: io_ply | 8.02 | 1.39 | 5.78x | 196.2 MiB | 179.1 MiB | 1.10x | 25.1 MiB |
| point_density | reference fixture: filter_point_density | 12.41 | 1.00 | 12.41x | 208.1 MiB | 177.6 MiB | 1.17x | 37.0 MiB |
| point_to_cell_data | reference fixture: filter_point_to_cell_data | 2.23 | 0.214 | 10.40x | 7.8 MiB | 182.2 MiB | 0.04x | 1.6 MiB |
| poly_data_distance | reference fixture: filter_poly_data_distance | 227.3 | 30.46 | 7.46x | 277.7 MiB | 182.3 MiB | 1.52x | 110.7 MiB |
| probe_100 | reference fixture: filter_probe | 7.52 | 0.538 | 13.96x | 201.5 MiB | 179.7 MiB | 1.12x | 27.2 MiB |
| quadric_clustering | reference fixture: filter_quadric_clustering | 2.93 | 1.07 | 2.72x | 207.8 MiB | 185.0 MiB | 1.12x | 13.8 MiB |
| quadric_clustering_large | reference fixture: filter_quadric_clustering | 49.92 | 7.20 | 6.93x | 267.5 MiB | 189.7 MiB | 1.41x | 71.3 MiB |
| quadric_decimate_50 | reference fixture: filter_decimate_50 | 37.75 | 3.83 | 9.85x | 240.3 MiB | 180.1 MiB | 1.33x | 65.4 MiB |
| quadric_decimate_50_large | reference fixture: filter_decimate_50 | 439.5 | 74.28 | 5.92x | 289.3 MiB | 192.1 MiB | 1.51x | 92.3 MiB |
| reflect | reference fixture: filter_reflect | 4.24 | 4.13 | 1.03x | 220.3 MiB | 179.9 MiB | 1.22x | 18.5 MiB |
| reflect_large | reference fixture: filter_reflect | 74.33 | 23.24 | 3.20x | 275.0 MiB | 183.6 MiB | 1.50x | 71.6 MiB |
| reverse_sense | reference fixture: filter_reverse_sense | 0.815 | 0.090 | 9.08x | 207.8 MiB | 178.3 MiB | 1.17x | 1.9 MiB |
| reverse_sense_large | reference fixture: filter_reverse_sense | 14.12 | 1.26 | 11.24x | 230.0 MiB | 181.1 MiB | 1.27x | 21.9 MiB |
| ribbon | reference fixture: filter_ribbon | 0.056 | 0.105 | 0.54x | 207.8 MiB | 183.7 MiB | 1.13x | 0 KiB |
| rotation_extrude | validation test: filter_rotation_extrude_test | 0.148 | 0.215 | 0.69x | 209.6 MiB | 178.1 MiB | 1.18x | 1.9 MiB |
| ruled_surface | reference fixture: filter_ruled_surface | 0.057 | 1.00 | 0.06x | 209.6 MiB | 178.3 MiB | 1.18x | 320 KiB |
| separate_cells | reference fixture: filter_separate_cells | 2.40 | 0.806 | 2.97x | 224.7 MiB | 179.1 MiB | 1.25x | 15.1 MiB |
| shrink | reference fixture: filter_shrink | 2.37 | 0.648 | 3.65x | 219.1 MiB | 178.6 MiB | 1.23x | 8.8 MiB |
| shrink_large | reference fixture: filter_shrink | 48.81 | 10.60 | 4.60x | 272.5 MiB | 180.9 MiB | 1.51x | 54.7 MiB |
| signed_distance_32 | reference fixture: filter_signed_distance | 223.8 | 188.8 | 1.19x | 260.5 MiB | 180.6 MiB | 1.44x | 66.3 MiB |
| silhouette | reference fixture: filter_silhouette | 10.46 | 1.00 | 10.46x | 232.2 MiB | 179.7 MiB | 1.29x | 12.2 MiB |
| slice | reference fixture: filter_slice | 2.39 | 0.442 | 5.39x | 206.2 MiB | 183.7 MiB | 1.12x | 5.0 MiB |
| slice_large | reference fixture: filter_slice | 19.67 | 1.15 | 17.11x | 7.2 MiB | 187.2 MiB | 0.04x | 2.8 MiB |
| smooth_20 | reference fixture: filter_smooth_20 | 25.95 | 1.15 | 22.62x | 19.8 MiB | 179.2 MiB | 0.11x | 20.9 MiB |
| smooth_20_constrained | reference fixture: filter_smooth_20 | 22.60 | 1.15 | 19.72x | 19.2 MiB | 181.0 MiB | 0.11x | 20.6 MiB |
| smooth_20_large | reference fixture: filter_smooth_20 | 347.4 | 19.41 | 17.90x | 21.0 MiB | 180.9 MiB | 0.12x | 22.2 MiB |
| smooth_50 | reference fixture: filter_smooth_50 | 46.83 | 1.81 | 25.89x | 19.8 MiB | 179.1 MiB | 0.11x | 20.9 MiB |
| smooth_50_large | reference fixture: filter_smooth_50 | 601.2 | 30.90 | 19.46x | 21.0 MiB | 181.1 MiB | 0.12x | 16.0 MiB |
| sphere_128x128 | reference fixture: source_sphere_128x128 | 12.40 | 1.31 | 9.46x | 258.1 MiB | 178.8 MiB | 1.44x | 25.9 MiB |
| sphere_32x32 | reference fixture: source_sphere_32x32 | 0.766 | 0.093 | 8.20x | 234.4 MiB | 177.6 MiB | 1.32x | 2.2 MiB |
| sphere_64x64 | reference fixture: source_sphere_64x64 | 3.13 | 0.364 | 8.58x | 241.0 MiB | 177.5 MiB | 1.36x | 6.6 MiB |
| spline | reference fixture: filter_spline | 0.023 | 0.043 | 0.54x | 239.1 MiB | 178.1 MiB | 1.34x | 0 KiB |
| stl_large | reference fixture: io_stl | 231.5 | 22.58 | 10.25x | 280.5 MiB | 184.0 MiB | 1.52x | 46.0 MiB |
| stl_roundtrip | reference fixture: io_stl | 19.22 | 1.63 | 11.79x | 255.6 MiB | 179.8 MiB | 1.42x | 21.2 MiB |
| subdivide_1 | reference fixture: filter_subdivide_1 | 45.07 | 3.33 | 13.52x | 272.2 MiB | 181.2 MiB | 1.50x | 34.0 MiB |
| subdivide_2 | reference fixture: filter_subdivide_2 | 230.4 | 16.12 | 14.29x | 274.6 MiB | 189.7 MiB | 1.45x | 35.0 MiB |
| surface_nets_32 | reference fixture: filter_surface_nets_32 | 69.43 | 16.78 | 4.14x | 276.2 MiB | 180.6 MiB | 1.53x | 27.5 MiB |
| texture_map_sphere | reference fixture: filter_texture_map_sphere | 0.435 | 0.185 | 2.35x | 259.4 MiB | 177.8 MiB | 1.46x | 1.2 MiB |
| threshold | reference fixture: filter_threshold | 2.38 | 0.198 | 12.01x | 259.4 MiB | 179.8 MiB | 1.44x | 0 KiB |
| topology_analysis | reference fixture: filter_topology_analysis | 5.04 | 0.722 | 6.98x | 254.4 MiB | 179.9 MiB | 1.41x | 640 KiB |
| topology_analysis_large | reference fixture: filter_topology_analysis | 89.86 | 8.45 | 10.64x | 276.9 MiB | 182.0 MiB | 1.52x | 17.8 MiB |
| transform | reference fixture: filter_transform_translate | 1.19 | 0.103 | 11.53x | 253.7 MiB | 179.7 MiB | 1.41x | 0 KiB |
| transform_large | reference fixture: filter_transform_large | 7.87 | 0.335 | 23.50x | 267.5 MiB | 189.7 MiB | 1.41x | 7.5 MiB |
| triangle_strips | reference fixture: filter_triangle_strips | 4.32 | 0.350 | 12.35x | 12.8 MiB | 178.5 MiB | 0.07x | 6.6 MiB |
| triangle_strips_large | reference fixture: filter_triangle_strips | 60.14 | 4.40 | 13.67x | 13.8 MiB | 180.9 MiB | 0.08x | 5.0 MiB |
| triangulate | reference fixture: filter_triangulate | 0.292 | 0.039 | 7.56x | 267.5 MiB | 177.6 MiB | 1.51x | 320 KiB |
| triangulate_large | reference fixture: filter_triangulate | 0.884 | 0.058 | 15.16x | 8.4 MiB | 179.5 MiB | 0.05x | 320 KiB |
| tube | reference fixture: filter_tube | 0.323 | 0.057 | 5.63x | 267.8 MiB | 178.4 MiB | 1.50x | 320 KiB |
| tube_large | reference fixture: filter_tube | 1.45 | 0.257 | 5.66x | 268.1 MiB | 192.3 MiB | 1.39x | 640 KiB |
| validate | reference fixture: filter_validate | 2.73 | 0.790 | 3.46x | 268.1 MiB | 177.9 MiB | 1.51x | 320 KiB |
| voronoi_200 | reference fixture: filter_voronoi_2d | 27.35 | 10.00 | 2.74x | 276.2 MiB | 179.1 MiB | 1.54x | 8.4 MiB |
| voxel_grid | reference fixture: filter_voxel_grid | 2.71 | 5.06 | 0.54x | 268.4 MiB | 177.4 MiB | 1.51x | 640 KiB |
| vtk_large | reference fixture: io_vtk_large | 378.0 | 65.87 | 5.74x | 289.3 MiB | 181.9 MiB | 1.59x | 23.2 MiB |
| vtk_roundtrip | reference fixture: io_vtk_roundtrip | 36.92 | 4.13 | 8.95x | 268.1 MiB | 179.7 MiB | 1.49x | 9.7 MiB |
| vtp_large | reference fixture: io_vtp | 441.4 | 58.33 | 7.57x | 279.9 MiB | 183.9 MiB | 1.52x | 22.9 MiB |
| vtp_roundtrip | reference fixture: io_vtp | 41.16 | 4.36 | 9.43x | 268.1 MiB | 181.2 MiB | 1.48x | 9.4 MiB |
| warp_scalar | reference fixture: filter_warp_scalar | 3.23 | 0.111 | 29.16x | 272.5 MiB | 179.7 MiB | 1.52x | 4.1 MiB |
| windowed_sinc_20 | reference fixture: filter_windowed_sinc | 19.35 | 1.38 | 14.07x | 267.8 MiB | 177.9 MiB | 1.50x | 5.0 MiB |

Benchmark command:

```bash
cargo test --test vtk_perf --features filters-smooth,filters-transform,filters-subdivide,filters-cell,filters-statistics,filters-texture,filters-flow,filters-boolean,filters-data,filters-distance -- --nocapture
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
