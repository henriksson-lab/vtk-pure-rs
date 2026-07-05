#!/usr/bin/env python3
"""Update README benchmark section from vtk_perf PERF_JSON output."""

import json
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
README = os.path.join(ROOT, "README.md")
PERF_TEST = os.path.join(ROOT, "tests", "vtk_perf.rs")
VALIDATION_TEST = os.path.join(ROOT, "tests", "vtk_validation.rs")
PERF_REF = os.path.join(ROOT, "tests", "vtk_validation", "reference", "perf_vtk_cpp.json")
REF_DIR = os.path.join(ROOT, "tests", "vtk_validation", "reference")


def read_perf_json(path):
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            if "PERF_JSON " not in line:
                continue
            payload = line.split("PERF_JSON ", 1)[1].strip()
            rows.append(json.loads(payload))
    deduped = {}
    for row in rows:
        deduped[row["algorithm"]] = row
    return [deduped[key] for key in sorted(deduped)]


def read_test_keys():
    with open(PERF_TEST, encoding="utf-8") as f:
        source = f.read()
    return re.findall(r'perf_test(?:_setup)?!\([^,]+,\s*"([^"]+)"', source)


def read_vtk_refs():
    with open(PERF_REF, encoding="utf-8") as f:
        raw = json.load(f)
    refs = {}
    for key, value in raw.items():
        if isinstance(value, dict):
            refs[key] = {
                "vtk_ms": value.get("time_s", value.get("seconds", 0.0)) * 1000.0,
                "vtk_rss_kib": value.get("rss_kib") or value.get("peak_rss_kib"),
            }
        else:
            refs[key] = {"vtk_ms": float(value) * 1000.0, "vtk_rss_kib": None}
    return refs


def add_uncaptured_rows(rows):
    by_key = {row["algorithm"]: row for row in rows}
    refs = read_vtk_refs()
    for key, row in by_key.items():
        vtk_ref = refs.get(key)
        if not vtk_ref:
            continue
        row["vtk_ms"] = vtk_ref.get("vtk_ms", row.get("vtk_ms"))
        row["vtk_rss_kib"] = vtk_ref.get("vtk_rss_kib")
        rust_rss = row.get("rust_rss_kib")
        vtk_rss = row.get("vtk_rss_kib")
        row["rss_ratio"] = (rust_rss / max(vtk_rss, 1)) if rust_rss is not None and vtk_rss is not None else None
    for key in read_test_keys():
        if key in by_key:
            continue
        vtk_ref = refs.get(key, {})
        by_key[key] = {
            "algorithm": key,
            "parity": "not-captured",
            "rust_ms": None,
            "vtk_ms": vtk_ref.get("vtk_ms"),
            "speed_ratio": None,
            "rust_rss_kib": None,
            "rust_peak_delta_kib": None,
            "vtk_rss_kib": vtk_ref.get("vtk_rss_kib"),
            "rss_ratio": None,
        }
    return [by_key[key] for key in sorted(by_key)]


def parity_reference_candidates(key):
    aliases = {
        "append_10_small": ["filter_append"],
        "append_3": ["filter_append"],
        "append_5_large": ["filter_append"],
        "arrow": ["source_arrow"],
        "boolean_union": ["filter_boolean_union"],
        "butterfly_1": ["filter_butterfly_1"],
        "byu_roundtrip": ["io_byu"],
        "calculator": ["filter_calculator"],
        "catmull_clark_1": ["filter_catmull_clark_1"],
        "cell_centers_large": ["filter_cell_centers"],
        "cell_quality_large": ["filter_cell_quality"],
        "cell_size_large": ["filter_cell_size"],
        "cell_to_point_data": ["filter_cell_to_point_data", "filter_cell_data_to_point_data"],
        "clean_3x_large": ["filter_clean"],
        "clean_large": ["filter_clean"],
        "clip": ["filter_clip_plane", "filter_clip_scalar"],
        "clip_large": ["filter_clip_plane", "filter_clip_scalar"],
        "cone_32": ["source_cone_32"],
        "contour_32": ["filter_contour"],
        "cube": ["source_cube"],
        "cylinder_32": ["source_cylinder_32"],
        "decimate_50_large": ["filter_decimate_50"],
        "delaunay_1000": ["filter_delaunay_2d", "filter_delaunay_2d_200pts"],
        "depth_sort_large": ["filter_depth_sort"],
        "disk_32": ["source_disk"],
        "elevation_large": ["filter_elevation"],
        "extract_edges_large": ["filter_extract_edges"],
        "fe_64": ["filter_fe_64", "filter_flying_edges"],
        "feature_edges_boundary": ["filter_feature_edges"],
        "feature_edges_large": ["filter_feature_edges"],
        "fill_holes_large": ["filter_fill_holes"],
        "glyph_10": ["filter_glyph"],
        "glyph_100": ["filter_glyph"],
        "glyph_50": ["filter_glyph"],
        "hausdorff_large": ["filter_hausdorff"],
        "linear_subdiv_1": ["filter_subdivide_1", "filter_subdivide_loop"],
        "mass_properties_large": ["filter_mass_properties"],
        "mc_64": ["filter_mc_64", "filter_marching_cubes_64"],
        "mirror_large": ["filter_mirror"],
        "normals_large": ["filter_normals"],
        "obj_large": ["io_obj"],
        "orient_normals": ["filter_orient_normals"],
        "pipeline_normals_smooth": ["filter_normals", "filter_smooth_50"],
        "plane_32": ["source_plane_32x32"],
        "ply_large": ["io_ply"],
        "ply_roundtrip": ["io_ply"],
        "point_to_cell_data": ["filter_point_to_cell_data", "filter_point_data_to_cell_data"],
        "probe_100": ["filter_probe"],
        "quadric_decimate_50": ["filter_decimate_50"],
        "quadric_decimate_50_large": ["filter_decimate_50"],
        "quadric_clustering_large": ["filter_quadric_clustering"],
        "reflect_large": ["filter_reflect"],
        "reverse_sense_large": ["filter_reverse_sense"],
        "shrink_large": ["filter_shrink"],
        "slice_large": ["filter_slice"],
        "signed_distance_32": ["filter_signed_distance"],
        "smooth_20_constrained": ["filter_smooth_20"],
        "smooth_20_large": ["filter_smooth_20"],
        "smooth_50_large": ["filter_smooth_50"],
        "sphere_128x128": ["source_sphere_128x128"],
        "sphere_32x32": ["source_sphere_32x32"],
        "stl_large": ["io_stl"],
        "stl_roundtrip": ["io_stl"],
        "subdivide_1": ["filter_subdivide_1", "filter_subdivide_loop"],
        "surface_nets_32": ["filter_surface_nets_32"],
        "texture_sphere": ["filter_texture_map_sphere"],
        "transform": ["filter_transform_translate"],
        "topology_analysis_large": ["filter_topology_analysis"],
        "triangle_strips_large": ["filter_triangle_strips"],
        "triangulate_large": ["filter_triangulate"],
        "tube_large": ["filter_tube"],
        "voronoi_200": ["filter_voronoi_2d"],
        "vtk_large": ["io_vtk_large"],
        "vtk_roundtrip": ["io_vtk_legacy", "io_vtk_roundtrip"],
        "vtp_large": ["io_vtp"],
        "vtp_roundtrip": ["io_vtp"],
        "windowed_sinc_20": ["filter_windowed_sinc"],
    }
    candidates = [key, f"filter_{key}", f"source_{key}", f"io_{key}"]
    candidates.extend(aliases.get(key, []))
    return candidates


def add_parity_labels(rows):
    refs = {
        os.path.splitext(name)[0]
        for name in os.listdir(REF_DIR)
        if name.endswith(".json") and name != "perf_vtk_cpp.json"
    }
    validation_aliases = {
        "orient_normals": ["filter_orient_consistent", "filter_auto_orient_normals"],
        "rotation_extrude": ["filter_rotation_extrude_test"],
    }
    with open(VALIDATION_TEST, encoding="utf-8") as f:
        validation_source = f.read()
    for row in rows:
        if row.get("parity") == "not-captured":
            continue
        match = next((name for name in parity_reference_candidates(row["algorithm"]) if name in refs), None)
        if match:
            row["parity"] = f"reference fixture: {match}"
            continue
        test_match = next(
            (
                name
                for name in validation_aliases.get(row["algorithm"], [])
                if re.search(rf"fn\s+{re.escape(name)}\s*\(", validation_source)
            ),
            None,
        )
        row["parity"] = f"validation test: {test_match}" if test_match else "reference missing"
    return rows


def fmt_ms(value):
    if value is None:
        return "n/a"
    if value < 1:
        return f"{value:.3f}"
    if value < 100:
        return f"{value:.2f}"
    return f"{value:.1f}"


def fmt_ratio(value):
    return "n/a" if value is None else f"{value:.2f}x"


def fmt_kib(value):
    if value is None:
        return "n/a"
    if value >= 1024 * 1024:
        return f"{value / (1024 * 1024):.2f} GiB"
    if value >= 1024:
        return f"{value / 1024:.1f} MiB"
    return f"{value} KiB"


def avg(values):
    values = [v for v in values if v is not None]
    return sum(values) / len(values) if values else None


def parity_label(value):
    if not value:
        return "n/a"
    return {
        "covered-by-vtk_validation": "covered",
        "not-captured": "not captured",
        "passed": "passed",
        "reference": "reference",
    }.get(value, value)


def build_section(rows):
    ratios = [row["speed_ratio"] for row in rows]
    rss_ratios = [row.get("rss_ratio") for row in rows]
    comparable_count = len([r for r in ratios if r is not None])
    faster = sum(1 for r in ratios if r is not None and r < 1.0)
    within_2x = sum(1 for r in ratios if r is not None and 1.0 <= r < 2.0)
    two_to_3x = sum(1 for r in ratios if r is not None and 2.0 <= r < 3.0)
    over_3x = sum(1 for r in ratios if r is not None and r >= 3.0)
    uncaptured = len(rows) - comparable_count
    rss_count = sum(1 for row in rows if row.get("rss_ratio") is not None)

    lines = [
        "## Performance vs VTK C++ 9.6",
        "",
        (
            f"Tracked {len(rows)} benchmark operations against VTK C++ 9.6; "
            f"{comparable_count} completed in the latest captured run. "
            f"Average speed ratio: **{fmt_ratio(avg(ratios))}** "
            "(Rust time / VTK time; lower is better)."
        ),
        "",
        (
            f"RSS is recorded from `/proc/self/status` where available. "
            f"Average RSS ratio over {rss_count} comparable operations: "
            f"**{fmt_ratio(avg(rss_ratios))}** (Rust RSS / VTK RSS; lower is better)."
        ),
        "",
        "| Category | Count | % |",
        "|---|---:|---:|",
        f"| Faster than C++ | {faster} | {faster / comparable_count * 100:.0f}% |",
        f"| Within 2x | {within_2x} | {within_2x / comparable_count * 100:.0f}% |",
        f"| 2-3x slower | {two_to_3x} | {two_to_3x / comparable_count * 100:.0f}% |",
        f"| >3x slower | {over_3x} | {over_3x / comparable_count * 100:.0f}% |",
        f"| Not captured in latest run | {uncaptured} | n/a |",
        "",
        "| Algorithm | Parity | Rust ms | VTK ms | Speed ratio | Rust RSS | VTK RSS | RSS ratio | Rust peak delta |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|",
    ]

    for row in rows:
        lines.append(
            "| {algorithm} | {parity} | {rust_ms} | {vtk_ms} | {speed_ratio} | "
            "{rust_rss} | {vtk_rss} | {rss_ratio} | {peak_delta} |".format(
                algorithm=row["algorithm"],
                parity=parity_label(row.get("parity")),
                rust_ms=fmt_ms(row.get("rust_ms")),
                vtk_ms=fmt_ms(row.get("vtk_ms")),
                speed_ratio=fmt_ratio(row.get("speed_ratio")),
                rust_rss=fmt_kib(row.get("rust_rss_kib")),
                vtk_rss=fmt_kib(row.get("vtk_rss_kib")),
                rss_ratio=fmt_ratio(row.get("rss_ratio")),
                peak_delta=fmt_kib(row.get("rust_peak_delta_kib")),
            )
        )

    lines.extend(
        [
            "",
            "Benchmark command:",
            "",
            "```bash",
            "cargo test --test vtk_perf --features filters-smooth,filters-transform,filters-subdivide,filters-cell,filters-statistics,filters-texture,filters-flow,filters-boolean,filters-data,filters-distance -- --nocapture",
            "```",
        ]
    )
    return "\n".join(lines)


def replace_section(readme, section):
    pattern = re.compile(
        r"^## Performance vs VTK C\+\+ 9\.6\n.*?(?=^## Test Coverage\n)",
        re.MULTILINE | re.DOTALL,
    )
    updated, count = pattern.subn(section + "\n\n", readme)
    if count != 1:
        raise SystemExit("could not find exactly one README performance section")
    return updated


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: update_benchmark_readme.py <vtk_perf_output.txt>")
    rows = add_parity_labels(add_uncaptured_rows(read_perf_json(sys.argv[1])))
    if not rows:
        raise SystemExit("no PERF_JSON rows found")
    with open(README, encoding="utf-8") as f:
        readme = f.read()
    section = build_section(rows)
    with open(README, "w", encoding="utf-8") as f:
        f.write(replace_section(readme, section))
    print(f"updated README with {len(rows)} benchmark rows")


if __name__ == "__main__":
    main()
