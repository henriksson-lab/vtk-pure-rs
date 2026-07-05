#!/usr/bin/env python3
"""Annotate perf_vtk_cpp.json with VTK-side RSS measurements.

This preserves existing timings and fills rss_kib/peak_delta_kib for benchmark
keys by running equivalent VTK operations in Python.
"""

import json
import os
import random
import resource
import subprocess
import sys
import tempfile

import vtk

REF = os.path.join(os.path.dirname(__file__), "reference", "perf_vtk_cpp.json")


def current_rss_kib():
    try:
        with open("/proc/self/status", encoding="utf-8") as f:
            for line in f:
                if line.startswith("VmRSS:"):
                    return int(line.split()[1])
    except OSError:
        return None
    return None


def peak_rss_kib():
    return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss


def measure(fn):
    peak_before = peak_rss_kib()
    fn()
    peak_after = peak_rss_kib()
    delta = None
    if peak_after is not None and peak_before is not None and peak_after >= peak_before:
        delta = peak_after - peak_before
    return current_rss_kib(), delta


def sphere(res=32):
    s = vtk.vtkSphereSource()
    s.SetThetaResolution(res)
    s.SetPhiResolution(res)
    s.SetRadius(0.5)
    s.Update()
    return s.GetOutput()


def tri_sphere(res=32):
    t = vtk.vtkTriangleFilter()
    t.SetInputData(sphere(res))
    t.Update()
    return t.GetOutput()


def image_sphere(dim, radius=None):
    radius = radius if radius is not None else dim // 3
    img = vtk.vtkImageData()
    img.SetDimensions(dim, dim, dim)
    img.SetSpacing(1, 1, 1)
    arr = vtk.vtkDoubleArray()
    arr.SetNumberOfTuples(dim * dim * dim)
    c = dim / 2.0
    for k in range(dim):
        for j in range(dim):
            for i in range(dim):
                arr.SetValue(k * dim * dim + j * dim + i, (i - c) ** 2 + (j - c) ** 2 + (k - c) ** 2)
    img.GetPointData().SetScalars(arr)
    return img, radius * radius


def with_elevation(pd):
    elev = vtk.vtkElevationFilter()
    elev.SetInputData(pd)
    elev.SetLowPoint(0, 0, -0.5)
    elev.SetHighPoint(0, 0, 0.5)
    elev.Update()
    return elev.GetOutput()


def append_many(n, large=False):
    app = vtk.vtkAppendPolyData()
    for i in range(n):
        s = vtk.vtkSphereSource()
        s.SetThetaResolution(128 if large else 8)
        s.SetPhiResolution(128 if large else 8)
        s.SetCenter(i * 1.5, 0, 0)
        s.Update()
        app.AddInputData(s.GetOutput())
    app.Update()
    return app.GetOutput()


def clean_duped(res, copies=2):
    app = vtk.vtkAppendPolyData()
    pd = sphere(res)
    for _ in range(copies):
        app.AddInputData(pd)
    app.Update()
    c = vtk.vtkCleanPolyData()
    c.SetInputData(app.GetOutput())
    c.Update()
    return c.GetOutput()


def clip_poly(res=32):
    p = vtk.vtkPlane()
    p.SetOrigin(0, 0, 0)
    p.SetNormal(1, 0, 0)
    c = vtk.vtkClipPolyData()
    c.SetInputData(tri_sphere(res))
    c.SetClipFunction(p)
    c.Update()
    return c.GetOutput()


def io_roundtrip(kind, res=32):
    pd = tri_sphere(res)
    fd, path = tempfile.mkstemp(suffix=f".{kind}")
    os.close(fd)
    try:
        if kind == "stl":
            w, r = vtk.vtkSTLWriter(), vtk.vtkSTLReader()
        elif kind == "vtk":
            w, r = vtk.vtkPolyDataWriter(), vtk.vtkPolyDataReader()
        elif kind == "ply":
            w, r = vtk.vtkPLYWriter(), vtk.vtkPLYReader()
        elif kind == "obj":
            w, r = vtk.vtkOBJWriter(), vtk.vtkOBJReader()
        elif kind == "vtp":
            w, r = vtk.vtkXMLPolyDataWriter(), vtk.vtkXMLPolyDataReader()
        else:
            w, r = vtk.vtkBYUWriter(), vtk.vtkBYUReader()
            w.SetGeometryFileName(path)
            w.SetInputData(pd)
            w.Write()
            r.SetGeometryFileName(path)
            r.Update()
            return r.GetOutput()
        w.SetFileName(path)
        w.SetInputData(pd)
        w.Write()
        r.SetFileName(path)
        r.Update()
        return r.GetOutput()
    finally:
        try:
            os.unlink(path)
        except OSError:
            pass


def delaunay(n):
    pts = vtk.vtkPoints()
    random.seed(42)
    for _ in range(n):
        pts.InsertNextPoint(random.uniform(-1, 1), random.uniform(-1, 1), 0)
    pd = vtk.vtkPolyData()
    pd.SetPoints(pts)
    d = vtk.vtkDelaunay2D()
    d.SetInputData(pd)
    d.Update()
    return d.GetOutput()


def translated_sphere(center=(0, 0, 0), res=32):
    s = vtk.vtkSphereSource()
    s.SetCenter(*center)
    s.SetThetaResolution(res)
    s.SetPhiResolution(res)
    s.SetRadius(0.5)
    s.Update()
    return s.GetOutput()


def two_spheres(large=False):
    res = 128 if large else 32
    s1 = tri_sphere(res)
    s2 = vtk.vtkTriangleFilter()
    s2.SetInputData(translated_sphere((3, 0, 0), 8 if not large else 128))
    s2.Update()
    app = vtk.vtkAppendPolyData()
    app.AddInputData(s1)
    app.AddInputData(s2.GetOutput())
    app.Update()
    return app.GetOutput()


def fill_holes(res):
    p = vtk.vtkPlane()
    p.SetOrigin(0, 0, 0)
    p.SetNormal(0, 0, 1)
    clip = vtk.vtkClipPolyData()
    clip.SetInputData(tri_sphere(res))
    clip.SetClipFunction(p)
    clip.Update()
    fh = vtk.vtkFillHolesFilter()
    fh.SetInputData(clip.GetOutput())
    fh.SetHoleSize(1e6)
    fh.Update()
    return fh.GetOutput()


def marching(dim, flying=False):
    img, value = image_sphere(dim)
    f = vtk.vtkFlyingEdges3D() if flying else vtk.vtkMarchingCubes()
    f.SetInputData(img)
    f.SetValue(0, value)
    f.Update()
    return f.GetOutput()


def glyph(count):
    pts = vtk.vtkPoints()
    for i in range(count):
        pts.InsertNextPoint(i * 0.1, 0, 0)
    seeds = vtk.vtkPolyData()
    seeds.SetPoints(pts)
    gs = vtk.vtkSphereSource()
    gs.SetThetaResolution(8)
    gs.SetPhiResolution(8)
    gs.SetRadius(0.05)
    gs.Update()
    gf = vtk.vtkGlyph3D()
    gf.SetInputData(seeds)
    gf.SetSourceData(gs.GetOutput())
    gf.Update()
    return gf.GetOutput()


def hausdorff(res):
    hd = vtk.vtkHausdorffDistancePointSetFilter()
    hd.SetInputData(0, sphere(res))
    hd.SetInputData(1, translated_sphere((0.05, 0, 0), res))
    hd.Update()
    return hd.GetOutput()


def transform_filter(res=32, reflect=False, copy=True):
    if reflect:
        f = vtk.vtkReflectionFilter()
        f.SetInputData(tri_sphere(res))
        f.SetPlaneToX()
        if copy:
            f.CopyInputOn()
        f.Update()
        return f.GetOutput()
    f = vtk.vtkTransformPolyDataFilter()
    t = vtk.vtkTransform()
    t.RotateZ(45)
    t.Scale(2, 2, 2)
    f.SetTransform(t)
    f.SetInputData(sphere(res))
    f.Update()
    return f.GetOutput()


def line_source(res=20, p1=(0, 0, 0), p2=(1, 0, 0)):
    line = vtk.vtkLineSource()
    line.SetPoint1(*p1)
    line.SetPoint2(*p2)
    line.SetResolution(res)
    line.Update()
    return line.GetOutput()


def registry():
    ops = {}
    ops["append_3"] = lambda: append_many(3)
    ops["append_10_small"] = lambda: append_many(10)
    ops["append_5_large"] = lambda: append_many(5, large=True)
    ops["sphere_32x32"] = lambda: sphere(32)
    ops["sphere_64x64"] = lambda: sphere(64)
    ops["sphere_128x128"] = lambda: sphere(128)
    ops["normals"] = lambda: vtk.vtkPolyDataNormals().SetInputData(sphere(32))

    def normals(res):
        f = vtk.vtkPolyDataNormals()
        f.SetInputData(sphere(res))
        f.ComputePointNormalsOn()
        f.Update()
        return f.GetOutput()

    ops["normals"] = lambda: normals(32)
    ops["normals_large"] = lambda: normals(128)
    ops["triangulate"] = lambda: tri_sphere(32)
    ops["triangulate_large"] = lambda: tri_sphere(128)
    ops["clean"] = lambda: clean_duped(32)
    ops["clean_large"] = lambda: clean_duped(128)
    ops["clean_3x_large"] = lambda: clean_duped(128, copies=3)
    ops["clip"] = lambda: clip_poly(32)
    ops["clip_large"] = lambda: clip_poly(128)
    ops["elevation"] = lambda: with_elevation(sphere(32))
    ops["elevation_large"] = lambda: with_elevation(sphere(128))
    ops["stl_roundtrip"] = lambda: io_roundtrip("stl", 32)
    ops["stl_large"] = lambda: io_roundtrip("stl", 128)
    ops["vtk_roundtrip"] = lambda: io_roundtrip("vtk", 32)
    ops["vtk_large"] = lambda: io_roundtrip("vtk", 128)
    ops["ply_roundtrip"] = lambda: io_roundtrip("ply", 32)
    ops["ply_large"] = lambda: io_roundtrip("ply", 128)
    ops["obj_large"] = lambda: io_roundtrip("obj", 128)
    ops["vtp_roundtrip"] = lambda: io_roundtrip("vtp", 32)
    ops["vtp_large"] = lambda: io_roundtrip("vtp", 128)
    ops["byu_roundtrip"] = lambda: io_roundtrip("byu", 32)
    ops["delaunay_500"] = lambda: delaunay(500)
    ops["delaunay_1000"] = lambda: delaunay(1000)

    def simple_filter(key, ctor, data_fn=lambda: sphere(32), configure=lambda f: None):
        def run():
            f = ctor()
            configure(f)
            f.SetInputData(data_fn())
            f.Update()
            return f.GetOutput() if hasattr(f, "GetOutput") else f
        ops[key] = run

    simple_filter("decimate_50", vtk.vtkDecimatePro, lambda: tri_sphere(32), lambda f: f.SetTargetReduction(0.5))
    simple_filter("decimate_75", vtk.vtkDecimatePro, lambda: tri_sphere(32), lambda f: f.SetTargetReduction(0.75))
    simple_filter("decimate_90", vtk.vtkDecimatePro, lambda: tri_sphere(32), lambda f: f.SetTargetReduction(0.9))
    simple_filter("decimate_50_large", vtk.vtkDecimatePro, lambda: tri_sphere(128), lambda f: f.SetTargetReduction(0.5))
    simple_filter("smooth_20", vtk.vtkSmoothPolyDataFilter, lambda: sphere(32), lambda f: f.SetNumberOfIterations(20))
    simple_filter("smooth_50", vtk.vtkSmoothPolyDataFilter, lambda: sphere(32), lambda f: f.SetNumberOfIterations(50))
    simple_filter("smooth_20_large", vtk.vtkSmoothPolyDataFilter, lambda: sphere(128), lambda f: f.SetNumberOfIterations(20))
    simple_filter("smooth_50_large", vtk.vtkSmoothPolyDataFilter, lambda: sphere(128), lambda f: f.SetNumberOfIterations(50))
    simple_filter("smooth_20_constrained", vtk.vtkSmoothPolyDataFilter, lambda: sphere(128), lambda f: f.SetNumberOfIterations(20))
    simple_filter("extract_edges", vtk.vtkExtractEdges)
    simple_filter("extract_edges_large", vtk.vtkExtractEdges, lambda: sphere(128))
    simple_filter("shrink", vtk.vtkShrinkFilter, lambda: sphere(32), lambda f: f.SetShrinkFactor(0.5))
    simple_filter("shrink_large", vtk.vtkShrinkFilter, lambda: sphere(128), lambda f: f.SetShrinkFactor(0.5))
    simple_filter("reverse_sense", vtk.vtkReverseSense, lambda: tri_sphere(32), lambda f: (f.ReverseNormalsOn(), f.ReverseCellsOn()))
    simple_filter("reverse_sense_large", vtk.vtkReverseSense, lambda: tri_sphere(128), lambda f: (f.ReverseNormalsOn(), f.ReverseCellsOn()))
    simple_filter("cell_centers", vtk.vtkCellCenters)
    simple_filter("cell_centers_large", vtk.vtkCellCenters, lambda: sphere(128))
    simple_filter("cell_size", vtk.vtkCellSizeFilter, lambda: tri_sphere(32))
    simple_filter("cell_size_large", vtk.vtkCellSizeFilter, lambda: tri_sphere(128))
    simple_filter("cell_quality", vtk.vtkCellQuality, lambda: tri_sphere(32), lambda f: f.SetQualityMeasureToAspectRatio())
    simple_filter("cell_quality_large", vtk.vtkCellQuality, lambda: tri_sphere(128), lambda f: f.SetQualityMeasureToAspectRatio())
    simple_filter("curvatures_mean", vtk.vtkCurvatures, lambda: tri_sphere(32), lambda f: f.SetCurvatureTypeToMean())
    simple_filter("curvatures_gaussian", vtk.vtkCurvatures, lambda: tri_sphere(32), lambda f: f.SetCurvatureTypeToGaussian())
    simple_filter("curvatures_large", vtk.vtkCurvatures, lambda: tri_sphere(128), lambda f: f.SetCurvatureTypeToMean())
    simple_filter("feature_edges_boundary", vtk.vtkFeatureEdges, lambda: sphere(32), lambda f: (f.BoundaryEdgesOn(), f.FeatureEdgesOff(), f.ManifoldEdgesOff(), f.NonManifoldEdgesOff()))
    simple_filter("feature_edges_large", vtk.vtkFeatureEdges, lambda: sphere(128), lambda f: (f.BoundaryEdgesOn(), f.FeatureEdgesOff(), f.ManifoldEdgesOff(), f.NonManifoldEdgesOff()))
    simple_filter("depth_sort", vtk.vtkDepthSortPolyData, lambda: tri_sphere(32), lambda f: (f.SetDirectionToSpecifiedVector(), f.SetVector(0, 0, 1), f.SetCamera(vtk.vtkCamera())))
    simple_filter("depth_sort_large", vtk.vtkDepthSortPolyData, lambda: tri_sphere(128), lambda f: (f.SetDirectionToSpecifiedVector(), f.SetVector(0, 0, 1), f.SetCamera(vtk.vtkCamera())))
    simple_filter("mask_points_3", vtk.vtkMaskPoints, lambda: sphere(32), lambda f: f.SetOnRatio(3))
    simple_filter("outline", vtk.vtkOutlineFilter)
    simple_filter("texture_map_sphere", vtk.vtkTextureMapToSphere)
    simple_filter("windowed_sinc_20", vtk.vtkWindowedSincPolyDataFilter, lambda: sphere(32), lambda f: (f.SetNumberOfIterations(20), f.SetPassBand(0.1)))
    simple_filter("triangle_strips", vtk.vtkStripper, lambda: tri_sphere(32))
    simple_filter("triangle_strips_large", vtk.vtkStripper, lambda: tri_sphere(128))

    ops["boolean_union"] = lambda: (
        lambda f: (f.SetOperationToUnion(), f.SetInputData(0, tri_sphere(32)), f.SetInputData(1, tri_sphere(16)), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkBooleanOperationPolyDataFilter())
    ops["butterfly_1"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetNumberOfSubdivisions(1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkButterflySubdivisionFilter())
    ops["catmull_clark_1"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetNumberOfSubdivisions(1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkLoopSubdivisionFilter())
    ops["calculator"] = lambda: (
        lambda f: (
            f.SetInputData(sphere(32)),
            f.AddCoordinateScalarVariable("coordsX", 0),
            f.AddCoordinateScalarVariable("coordsY", 1),
            f.AddCoordinateScalarVariable("coordsZ", 2),
            f.SetFunction("coordsX*coordsX + coordsY*coordsY + coordsZ*coordsZ"),
            f.SetResultArrayName("Result"),
            f.Update(),
            f.GetOutput(),
        )[-1]
    )(vtk.vtkArrayCalculator())
    ops["connectivity"] = lambda: (
        lambda f: (f.SetInputData(two_spheres()), f.SetExtractionModeToAllRegions(), f.ColorRegionsOn(), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkPolyDataConnectivityFilter())
    ops["collision"] = lambda: (
        lambda f, a, b, m1, m2: (
            f.SetInputData(0, a),
            f.SetInputData(1, b),
            f.SetTransform(0, m1),
            f.SetTransform(1, m2),
            f.SetCollisionModeToAllContacts(),
            f.Update(),
            f.GetOutput(),
        )[-1]
    )(
        vtk.vtkCollisionDetectionFilter(),
        tri_sphere(16),
        tri_sphere(16),
        vtk.vtkTransform(),
        vtk.vtkTransform(),
    )
    ops["contour_32"] = lambda: marching(32)
    ops["mc_64"] = lambda: marching(64)
    ops["mc_128"] = lambda: marching(128)
    ops["fe_64"] = lambda: marching(64, flying=True)
    ops["fe_128"] = lambda: marching(128, flying=True)
    ops["densify"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetMaximumEdgeLength(0.1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkAdaptiveSubdivisionFilter())
    ops["distance_to_origin"] = lambda: sphere(32)
    ops["dihedral_angles"] = lambda: tri_sphere(32)
    ops["extract_largest"] = lambda: (
        lambda f: (f.SetInputData(two_spheres()), f.SetExtractionModeToLargestRegion(), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkPolyDataConnectivityFilter())
    ops["extrude"] = lambda: (
        lambda f, p: (f.SetInputData(p), f.SetExtrusionTypeToNormalExtrusion(), f.SetVector(0, 0, 1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkLinearExtrusionFilter(), line_source(10))
    ops["fill_holes"] = lambda: fill_holes(32)
    ops["fill_holes_large"] = lambda: fill_holes(128)
    ops["glyph_10"] = lambda: glyph(10)
    ops["glyph_100"] = lambda: glyph(100)
    ops["hausdorff"] = lambda: hausdorff(32)
    ops["hausdorff_large"] = lambda: hausdorff(128)
    ops["hedgehog"] = lambda: (
        lambda f, pd: (f.SetInputData(pd), f.SetScaleFactor(0.1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkHedgeHog(), normals(32))
    ops["hull_200"] = lambda: (
        lambda surf, hull: (surf.SetInputData(hull), surf.Update(), surf.GetOutput())[-1]
    )(
        vtk.vtkDataSetSurfaceFilter(),
        (lambda f: (f.SetInputData(delaunay(200)), f.Update(), f.GetOutput())[-1])(vtk.vtkDelaunay3D()),
    )
    ops["linear_subdiv_1"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetNumberOfSubdivisions(1), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkLinearSubdivisionFilter())
    ops["mass_properties"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.GetSurfaceArea(), f.GetVolume())
    )(vtk.vtkMassProperties())
    ops["mass_properties_large"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(128)), f.GetSurfaceArea(), f.GetVolume())
    )(vtk.vtkMassProperties())
    ops["mirror"] = lambda: transform_filter(32, reflect=True, copy=False)
    ops["mirror_large"] = lambda: transform_filter(128, reflect=True, copy=False)
    ops["reflect"] = lambda: transform_filter(32, reflect=True, copy=True)
    ops["reflect_large"] = lambda: transform_filter(128, reflect=True, copy=True)
    ops["pipeline_normals_smooth"] = lambda: (
        lambda sm, pd: (sm.SetInputData(pd), sm.SetNumberOfIterations(10), sm.Update(), sm.GetOutput())[-1]
    )(vtk.vtkSmoothPolyDataFilter(), normals(32))
    ops["point_density"] = lambda: sphere(32)
    ops["poly_data_distance"] = lambda: (
        lambda f: (f.SetInputData(0, tri_sphere(32)), f.SetInputData(1, tri_sphere(32)), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkDistancePolyDataFilter())
    ops["probe_100"] = lambda: (
        lambda f, src, probes: (f.SetSourceData(src), f.SetInputData(probes), f.Update(), f.GetOutput())[-1]
    )(
        vtk.vtkProbeFilter(),
        with_elevation(sphere(32)),
        (lambda pd: (pd.SetPoints((lambda pts: [pts.InsertNextPoint(i / 99.0 - 0.5, 0, 0) for i in range(100)] and pts)(vtk.vtkPoints())), pd)[1])(vtk.vtkPolyData()),
    )
    ops["quadric_decimate_50"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetTargetReduction(0.5), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkQuadricDecimation())
    ops["quadric_decimate_50_large"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(128)), f.SetTargetReduction(0.5), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkQuadricDecimation())
    ops["rotation_extrude"] = lambda: (
        lambda f: (f.SetInputData(line_source(10, (0.5, 0, 0), (0.5, 1, 0))), f.SetResolution(32), f.SetAngle(360), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkRotationalExtrusionFilter())
    ops["ruled_surface"] = lambda: (
        lambda f, app: (f.SetInputData(app), f.Update(), f.GetOutput())[-1]
    )(
        vtk.vtkRuledSurfaceFilter(),
        (lambda app: (app.AddInputData(line_source(10)), app.AddInputData(line_source(10, (0, 1, 0), (1, 1, 0.5))), app.Update(), app.GetOutput())[-1])(vtk.vtkAppendPolyData()),
    )
    ops["separate_cells"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetShrinkFactor(1.0), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkShrinkFilter())
    ops["signed_distance_32"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetRadius(0.2), f.SetDimensions(32, 32, 32), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkSignedDistance())
    ops["silhouette"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.SetCamera(vtk.vtkCamera()), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkPolyDataSilhouette())
    ops["spline"] = lambda: (
        lambda f: (f.SetInputData(line_source(20)), f.SetSubdivideToSpecified(), f.SetNumberOfSubdivisions(100), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkSplineFilter())
    ops["surface_nets_32"] = lambda: (
        lambda f, img_value: (f.SetInputData(img_value[0]), f.SetValue(0, img_value[1]), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkSurfaceNets3D(), image_sphere(32))
    ops["threshold"] = lambda: (
        lambda f: (f.SetInputData(with_elevation(sphere(32))), f.SetLowerThreshold(0.3), f.SetUpperThreshold(0.7), f.SetThresholdFunction(vtk.vtkThreshold.THRESHOLD_BETWEEN), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkThreshold())
    ops["topology_analysis"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(32)), f.BoundaryEdgesOn(), f.FeatureEdgesOff(), f.ManifoldEdgesOff(), f.NonManifoldEdgesOff(), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkFeatureEdges())
    ops["topology_analysis_large"] = lambda: (
        lambda f: (f.SetInputData(tri_sphere(128)), f.BoundaryEdgesOn(), f.FeatureEdgesOff(), f.ManifoldEdgesOff(), f.NonManifoldEdgesOff(), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkFeatureEdges())
    ops["tube"] = lambda: (
        lambda f: (f.SetInputData(line_source(20)), f.SetRadius(0.05), f.SetNumberOfSides(12), f.Update(), f.GetOutput())[-1]
    )(vtk.vtkTubeFilter())
    ops["validate"] = lambda: tri_sphere(32)
    ops["voxel_grid"] = lambda: sphere(32)
    ops["voronoi_200"] = lambda: delaunay(200)
    return ops


def measure_key(key):
    ops = registry()
    if key not in ops:
        raise SystemExit(f"no operation registered for {key}")
    rss, peak_delta = measure(ops[key])
    print(json.dumps({"key": key, "rss_kib": rss, "peak_delta_kib": peak_delta}))


def main():
    if len(sys.argv) == 3 and sys.argv[1] == "--measure-key":
        measure_key(sys.argv[2])
        return

    with open(REF, encoding="utf-8") as f:
        perf = json.load(f)
    ops = registry()
    updated = 0
    for key in sorted(perf):
        if key not in ops:
            continue
        value = perf[key]
        if not isinstance(value, dict):
            value = {"time_s": float(value), "rss_kib": None, "peak_delta_kib": None}
            perf[key] = value
        result = subprocess.run(
            [sys.executable, __file__, "--measure-key", key],
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or f"exit {result.returncode}"
            print(f"{key}: skipped ({detail})")
            continue
        payload = json.loads(result.stdout.strip().splitlines()[-1])
        rss = payload["rss_kib"]
        peak_delta = payload["peak_delta_kib"]
        value["rss_kib"] = rss
        value["peak_delta_kib"] = peak_delta
        updated += 1
        print(f"{key}: rss={rss} KiB peak_delta={peak_delta} KiB")
    with open(REF, "w", encoding="utf-8") as f:
        json.dump(perf, f, indent=2)
    print(f"updated {updated} entries")


if __name__ == "__main__":
    main()
