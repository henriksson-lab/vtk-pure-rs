#!/usr/bin/env python3
"""Generate VTK C++ performance reference timings.

Reads the existing perf_vtk_cpp.json, adds NEW timing entries for operations
not already present, and writes back the merged JSON.

Each timing runs the VTK C++ operation 50 times and takes the median.
"""

import vtk
import json
import time
import os
import tempfile

REF_DIR = os.path.join(os.path.dirname(__file__), "reference")


def bench(fn, n=50):
    """Run fn n times, return median time in seconds."""
    fn()  # warmup
    times = []
    for _ in range(n):
        t0 = time.perf_counter()
        fn()
        times.append(time.perf_counter() - t0)
    times.sort()
    return times[len(times) // 2]


def make_sphere(theta=32, phi=32):
    s = vtk.vtkSphereSource()
    s.SetThetaResolution(theta)
    s.SetPhiResolution(phi)
    s.SetRadius(0.5)
    s.Update()
    return s.GetOutput()


def make_sphere_128():
    return make_sphere(128, 128)


# Load existing
perf_path = os.path.join(REF_DIR, "perf_vtk_cpp.json")
if os.path.exists(perf_path):
    with open(perf_path) as f:
        perf = json.load(f)
else:
    perf = {}

print("=== Generating NEW VTK C++ performance references ===")

# --- Sources ---
# cone
def bench_cone():
    c = vtk.vtkConeSource()
    c.SetResolution(32)
    c.Update()
    return c.GetOutput()
perf["cone_32"] = bench(lambda: bench_cone())
print(f"  cone_32: {perf['cone_32']*1000:.3f}ms")

# cube
def bench_cube():
    c = vtk.vtkCubeSource()
    c.Update()
    return c.GetOutput()
perf["cube"] = bench(lambda: bench_cube())
print(f"  cube: {perf['cube']*1000:.3f}ms")

# cylinder
def bench_cylinder():
    c = vtk.vtkCylinderSource()
    c.SetResolution(32)
    c.Update()
    return c.GetOutput()
perf["cylinder_32"] = bench(lambda: bench_cylinder())
print(f"  cylinder_32: {perf['cylinder_32']*1000:.3f}ms")

# arrow
def bench_arrow():
    a = vtk.vtkArrowSource()
    a.Update()
    return a.GetOutput()
perf["arrow"] = bench(lambda: bench_arrow())
print(f"  arrow: {perf['arrow']*1000:.3f}ms")

# plane
def bench_plane():
    p = vtk.vtkPlaneSource()
    p.SetResolution(32, 32)
    p.Update()
    return p.GetOutput()
perf["plane_32"] = bench(lambda: bench_plane())
print(f"  plane_32: {perf['plane_32']*1000:.3f}ms")

# disk
def bench_disk():
    d = vtk.vtkDiskSource()
    d.SetRadialResolution(8)
    d.SetCircumferentialResolution(32)
    d.Update()
    return d.GetOutput()
perf["disk_32"] = bench(lambda: bench_disk())
print(f"  disk_32: {perf['disk_32']*1000:.3f}ms")

# --- Filters ---

# warp_scalar (use sphere with elevation + normals)
sp32 = make_sphere(32, 32)
nf = vtk.vtkPolyDataNormals()
nf.SetInputData(sp32)
nf.Update()
sp_normals = nf.GetOutput()

ef = vtk.vtkElevationFilter()
ef.SetInputData(sp_normals)
ef.SetLowPoint(0, 0, -0.5)
ef.SetHighPoint(0, 0, 0.5)
ef.Update()
sp_elev = ef.GetOutput()

def bench_warp():
    wf = vtk.vtkWarpScalar()
    wf.SetInputData(sp_elev)
    wf.SetScaleFactor(0.1)
    wf.Update()
    return wf.GetOutput()
perf["warp_scalar"] = bench(lambda: bench_warp())
print(f"  warp_scalar: {perf['warp_scalar']*1000:.3f}ms")

# transform
def bench_transform():
    tf = vtk.vtkTransformPolyDataFilter()
    t = vtk.vtkTransform()
    t.RotateZ(45)
    t.Scale(2, 2, 2)
    tf.SetTransform(t)
    tf.SetInputData(sp32)
    tf.Update()
    return tf.GetOutput()
perf["transform"] = bench(lambda: bench_transform())
print(f"  transform: {perf['transform']*1000:.3f}ms")

# subdivide (Loop, 1 iteration)
tri = vtk.vtkTriangleFilter()
tri.SetInputData(sp32)
tri.Update()
sp32_tri = tri.GetOutput()

def bench_subdivide():
    sf = vtk.vtkLoopSubdivisionFilter()
    sf.SetInputData(sp32_tri)
    sf.SetNumberOfSubdivisions(1)
    sf.Update()
    return sf.GetOutput()
perf["subdivide_1"] = bench(lambda: bench_subdivide())
print(f"  subdivide_1: {perf['subdivide_1']*1000:.3f}ms")

# clip_closed
def bench_clip_closed():
    plane = vtk.vtkPlane()
    plane.SetOrigin(0, 0, 0)
    plane.SetNormal(1, 0, 0)
    cf = vtk.vtkClipClosedSurface()
    pc = vtk.vtkPlaneCollection()
    pc.AddItem(plane)
    cf.SetClippingPlanes(pc)
    cf.SetInputData(sp32_tri)
    cf.Update()
    return cf.GetOutput()
perf["clip_closed"] = bench(lambda: bench_clip_closed())
print(f"  clip_closed: {perf['clip_closed']*1000:.3f}ms")

# cell_data_to_point_data
# First add cell data
ef2 = vtk.vtkElevationFilter()
ef2.SetInputData(sp32)
ef2.Update()
sp_with_scalars = ef2.GetOutput()
p2c = vtk.vtkPointDataToCellData()
p2c.SetInputData(sp_with_scalars)
p2c.Update()
sp_cell_data = p2c.GetOutput()

def bench_cell_to_point():
    c2p = vtk.vtkCellDataToPointData()
    c2p.SetInputData(sp_cell_data)
    c2p.Update()
    return c2p.GetOutput()
perf["cell_to_point_data"] = bench(lambda: bench_cell_to_point())
print(f"  cell_to_point_data: {perf['cell_to_point_data']*1000:.3f}ms")

def bench_point_to_cell():
    p2c = vtk.vtkPointDataToCellData()
    p2c.SetInputData(sp_with_scalars)
    p2c.Update()
    return p2c.GetOutput()
perf["point_to_cell_data"] = bench(lambda: bench_point_to_cell())
print(f"  point_to_cell_data: {perf['point_to_cell_data']*1000:.3f}ms")

# PLY roundtrip
def bench_ply():
    td = tempfile.mkdtemp()
    path = os.path.join(td, "test.ply")
    w = vtk.vtkPLYWriter()
    w.SetFileName(path)
    w.SetInputData(sp32_tri)
    w.Write()
    r = vtk.vtkPLYReader()
    r.SetFileName(path)
    r.Update()
    return r.GetOutput()
perf["ply_roundtrip"] = bench(lambda: bench_ply())
print(f"  ply_roundtrip: {perf['ply_roundtrip']*1000:.3f}ms")

# slice
def bench_slice():
    plane = vtk.vtkPlane()
    plane.SetOrigin(0, 0, 0)
    plane.SetNormal(1, 0, 0)
    c = vtk.vtkCutter()
    c.SetCutFunction(plane)
    c.SetInputData(sp32_tri)
    c.Update()
    return c.GetOutput()
perf["slice"] = bench(lambda: bench_slice())
print(f"  slice: {perf['slice']*1000:.3f}ms")

# orient normals
def bench_orient():
    nf = vtk.vtkPolyDataNormals()
    nf.SetInputData(sp32_tri)
    nf.ConsistencyOn()
    nf.AutoOrientNormalsOn()
    nf.Update()
    return nf.GetOutput()
perf["orient_normals"] = bench(lambda: bench_orient())
print(f"  orient_normals: {perf['orient_normals']*1000:.3f}ms")

# center_of_mass
def bench_com():
    cf = vtk.vtkCenterOfMass()
    cf.SetInputData(sp32)
    cf.Update()
    return cf.GetCenter()
perf["center_of_mass"] = bench(lambda: bench_com())
print(f"  center_of_mass: {perf['center_of_mass']*1000:.3f}ms")

# ribbon from line
line = vtk.vtkLineSource()
line.SetPoint1(0, 0, 0)
line.SetPoint2(1, 0, 0)
line.SetResolution(20)
line.Update()
line_pd = line.GetOutput()

def bench_ribbon():
    rf = vtk.vtkRibbonFilter()
    rf.SetInputData(line_pd)
    rf.SetWidth(0.1)
    rf.Update()
    return rf.GetOutput()
perf["ribbon"] = bench(lambda: bench_ribbon())
print(f"  ribbon: {perf['ribbon']*1000:.3f}ms")

# quadric_clustering
def bench_quadric():
    qc = vtk.vtkQuadricClustering()
    qc.SetInputData(sp32_tri)
    qc.SetNumberOfXDivisions(10)
    qc.SetNumberOfYDivisions(10)
    qc.SetNumberOfZDivisions(10)
    qc.Update()
    return qc.GetOutput()
perf["quadric_clustering"] = bench(lambda: bench_quadric())
print(f"  quadric_clustering: {perf['quadric_clustering']*1000:.3f}ms")

# gradient
def bench_gradient():
    gf = vtk.vtkGradientFilter()
    gf.SetInputData(sp_with_scalars)
    gf.SetInputScalars(0, "Elevation")
    gf.Update()
    return gf.GetOutput()
perf["gradient"] = bench(lambda: bench_gradient())
print(f"  gradient: {perf['gradient']*1000:.3f}ms")

# extract_cells
def bench_extract_cells():
    ef = vtk.vtkExtractCells()
    ef.SetInputData(sp32_tri)
    ids = vtk.vtkIdList()
    for i in range(0, sp32_tri.GetNumberOfCells(), 2):
        ids.InsertNextId(i)
    ef.SetCellList(ids)
    ef.Update()
    return ef.GetOutput()
perf["extract_cells_half"] = bench(lambda: bench_extract_cells())
print(f"  extract_cells_half: {perf['extract_cells_half']*1000:.3f}ms")

# offset_surface (vtkWarpVector along normals -- approximate)
def bench_offset():
    wv = vtk.vtkWarpVector()
    wv.SetInputData(sp_normals)
    wv.SetInputArrayToProcess(0, 0, 0, 0, "Normals")
    wv.SetScaleFactor(0.1)
    wv.Update()
    return wv.GetOutput()
perf["offset_surface"] = bench(lambda: bench_offset())
print(f"  offset_surface: {perf['offset_surface']*1000:.3f}ms")

# ---- LARGE mesh versions ----
sp128 = make_sphere_128()
tri128 = vtk.vtkTriangleFilter()
tri128.SetInputData(sp128)
tri128.Update()
sp128_tri = tri128.GetOutput()

# slice large
def bench_slice_large():
    plane = vtk.vtkPlane()
    plane.SetOrigin(0, 0, 0)
    plane.SetNormal(1, 0, 0)
    c = vtk.vtkCutter()
    c.SetCutFunction(plane)
    c.SetInputData(sp128_tri)
    c.Update()
    return c.GetOutput()
perf["slice_large"] = bench(lambda: bench_slice_large())
print(f"  slice_large: {perf['slice_large']*1000:.3f}ms")

# subdivide large
def bench_subdiv_large():
    sf = vtk.vtkLoopSubdivisionFilter()
    sf.SetInputData(sp32_tri)  # sp32 not sp128 -- subdivision blows up on 128
    sf.SetNumberOfSubdivisions(2)
    sf.Update()
    return sf.GetOutput()
perf["subdivide_2"] = bench(lambda: bench_subdiv_large())
print(f"  subdivide_2: {perf['subdivide_2']*1000:.3f}ms")

# transform large
def bench_transform_large():
    tf = vtk.vtkTransformPolyDataFilter()
    t = vtk.vtkTransform()
    t.RotateZ(45)
    t.Scale(2, 2, 2)
    tf.SetTransform(t)
    tf.SetInputData(sp128)
    tf.Update()
    return tf.GetOutput()
perf["transform_large"] = bench(lambda: bench_transform_large())
print(f"  transform_large: {perf['transform_large']*1000:.3f}ms")

# quadric_clustering large
def bench_quadric_large():
    qc = vtk.vtkQuadricClustering()
    qc.SetInputData(sp128_tri)
    qc.SetNumberOfXDivisions(20)
    qc.SetNumberOfYDivisions(20)
    qc.SetNumberOfZDivisions(20)
    qc.Update()
    return qc.GetOutput()
perf["quadric_clustering_large"] = bench(lambda: bench_quadric_large())
print(f"  quadric_clustering_large: {perf['quadric_clustering_large']*1000:.3f}ms")

# connectivity large
def bench_connectivity_large():
    cf = vtk.vtkConnectivityFilter()
    cf.SetInputData(sp128_tri)
    cf.SetExtractionModeToAllRegions()
    cf.ColorRegionsOn()
    cf.Update()
    return cf.GetOutput()
perf["connectivity_large"] = bench(lambda: bench_connectivity_large())
print(f"  connectivity_large: {perf['connectivity_large']*1000:.3f}ms")

# glyph large (50 glyphs)
def bench_glyph_large():
    pts = vtk.vtkPoints()
    for i in range(50):
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
perf["glyph_50"] = bench(lambda: bench_glyph_large())
print(f"  glyph_50: {perf['glyph_50']*1000:.3f}ms")

# PLY large roundtrip
def bench_ply_large():
    td = tempfile.mkdtemp()
    path = os.path.join(td, "test_large.ply")
    w = vtk.vtkPLYWriter()
    w.SetFileName(path)
    w.SetInputData(sp128_tri)
    w.Write()
    r = vtk.vtkPLYReader()
    r.SetFileName(path)
    r.Update()
    return r.GetOutput()
perf["ply_large"] = bench(lambda: bench_ply_large())
print(f"  ply_large: {perf['ply_large']*1000:.3f}ms")

# tube large (100 segments)
def bench_tube_large():
    line = vtk.vtkLineSource()
    line.SetPoint1(0, 0, 0)
    line.SetPoint2(1, 0, 0)
    line.SetResolution(100)
    line.Update()
    tf = vtk.vtkTubeFilter()
    tf.SetInputData(line.GetOutput())
    tf.SetRadius(0.05)
    tf.SetNumberOfSides(12)
    tf.Update()
    return tf.GetOutput()
perf["tube_large"] = bench(lambda: bench_tube_large())
print(f"  tube_large: {perf['tube_large']*1000:.3f}ms")

# Save
with open(perf_path, "w") as f:
    json.dump(perf, f, indent=2)
print(f"\nSaved {len(perf)} entries to {perf_path}")
