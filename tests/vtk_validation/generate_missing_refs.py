#!/usr/bin/env python3
"""Generate VTK C++ reference data for features missing validation tests."""
import json
import os

import vtk

OUT = os.path.join(os.path.dirname(__file__), "reference")
os.makedirs(OUT, exist_ok=True)

def sphere(res=32):
    s = vtk.vtkSphereSource()
    s.SetThetaResolution(res); s.SetPhiResolution(res)
    s.Update()
    return s.GetOutput()

def tri_sphere(res=32):
    s = sphere(res)
    t = vtk.vtkTriangleFilter(); t.SetInputData(s); t.Update()
    return t.GetOutput()

def save(name, data):
    with open(os.path.join(OUT, f"{name}.json"), "w") as f:
        json.dump(data, f, indent=2)
    print(f"  wrote {name}.json")

def bounds(pd):
    b = pd.GetBounds()
    return list(b)

def basic(pd):
    return {"num_points": pd.GetNumberOfPoints(), "num_cells": pd.GetNumberOfCells(), "bounds": bounds(pd)}

# --- Slice ---
print("slice...")
pd = tri_sphere(32)
plane = vtk.vtkPlane(); plane.SetOrigin(0,0,0); plane.SetNormal(1,0,0)
cutter = vtk.vtkCutter(); cutter.SetInputData(pd); cutter.SetCutFunction(plane); cutter.Update()
r = cutter.GetOutput()
save("filter_slice", {"num_points": r.GetNumberOfPoints(), "num_lines": r.GetNumberOfLines()})

# --- Fill Holes ---
print("fill_holes...")
pd = tri_sphere(32)
clip = vtk.vtkClipPolyData()
clip.SetInputData(pd)
p = vtk.vtkPlane(); p.SetOrigin(0,0,0); p.SetNormal(0,0,1)
clip.SetClipFunction(p); clip.Update()
clipped = clip.GetOutput()
fh = vtk.vtkFillHolesFilter(); fh.SetInputData(clipped); fh.SetHoleSize(1e6); fh.Update()
r = fh.GetOutput()
save("filter_fill_holes", {"num_points_before": clipped.GetNumberOfPoints(), "num_cells_before": clipped.GetNumberOfCells(),
     "num_points": r.GetNumberOfPoints(), "num_cells": r.GetNumberOfCells()})

# --- Cell Quality ---
print("cell_quality...")
pd = tri_sphere(32)
cq = vtk.vtkCellQuality(); cq.SetInputData(pd)
cq.SetQualityMeasureToAspectRatio(); cq.Update()
r = cq.GetOutput()
arr = r.GetCellData().GetArray("CellQuality")
vals = [arr.GetValue(i) for i in range(arr.GetNumberOfTuples())]
save("filter_cell_quality", {"num_cells": r.GetNumberOfCells(), "quality_min": min(vals), "quality_max": max(vals), "quality_mean": sum(vals)/len(vals)})

# --- Quadric Clustering ---
print("quadric_clustering...")
pd = tri_sphere(32)
qc = vtk.vtkQuadricClustering(); qc.SetInputData(pd)
qc.SetNumberOfXDivisions(10); qc.SetNumberOfYDivisions(10); qc.SetNumberOfZDivisions(10)
qc.Update()
r = qc.GetOutput()
save("filter_quadric_clustering", basic(r))

# --- Orient Normals ---
print("orient_normals...")
pd = tri_sphere(32)
# Flip some faces
pts = pd.GetPoints()
cells = pd.GetPolys()
# VTK doesn't have a simple orient-only filter, skip

# --- Gradient ---
print("gradient...")
pd = sphere(32)
elev = vtk.vtkElevationFilter(); elev.SetInputData(pd)
elev.SetLowPoint(0,0,-0.5); elev.SetHighPoint(0,0,0.5); elev.Update()
grad = vtk.vtkGradientFilter(); grad.SetInputData(elev.GetOutput())
grad.SetInputArrayToProcess(0,0,0,0,"Elevation"); grad.Update()
r = grad.GetOutput()
garr = r.GetPointData().GetArray("Gradients")
save("filter_gradient", {"num_points": r.GetNumberOfPoints(), "has_gradient_array": garr is not None,
     "gradient_components": garr.GetNumberOfComponents() if garr else 0})

# --- Triangle Strips ---
print("triangle_strips...")
pd = tri_sphere(32)
ts = vtk.vtkStripper(); ts.SetInputData(pd); ts.Update()
r = ts.GetOutput()
save("filter_triangle_strips", {"num_points": r.GetNumberOfPoints(), "num_strips": r.GetNumberOfStrips(),
     "num_polys": r.GetNumberOfPolys()})

# --- Cell Centers ---
print("cell_centers...")
pd = sphere(32)
cc = vtk.vtkCellCenters(); cc.SetInputData(pd); cc.Update()
r = cc.GetOutput()
save("filter_cell_centers", {"num_points": r.GetNumberOfPoints()})

# --- Mirror ---
print("mirror...")
pd = tri_sphere(32)
mi = vtk.vtkReflectionFilter(); mi.SetInputData(pd); mi.SetPlaneToX(); mi.Update()
r = mi.GetOutput()
save("filter_mirror", {"num_points": r.GetNumberOfPoints(), "num_cells": r.GetNumberOfCells()})

# --- Outline ---
print("outline...")
pd = sphere(32)
ol = vtk.vtkOutlineFilter(); ol.SetInputData(pd); ol.Update()
r = ol.GetOutput()
save("filter_outline", {"num_points": r.GetNumberOfPoints(), "num_lines": r.GetNumberOfLines()})

# --- Extrude ---
print("extrude...")
plane_src = vtk.vtkPlaneSource(); plane_src.SetResolution(10,10); plane_src.Update()
ex = vtk.vtkLinearExtrusionFilter(); ex.SetInputData(plane_src.GetOutput())
ex.SetExtrusionTypeToNormalExtrusion(); ex.SetVector(0,0,1); ex.SetScaleFactor(1.0)
ex.Update()
r = ex.GetOutput()
save("filter_extrude", {"num_points": r.GetNumberOfPoints(), "num_cells": r.GetNumberOfCells()})

# --- Windowed Sinc ---
print("windowed_sinc...")
pd = sphere(32)
ws = vtk.vtkWindowedSincPolyDataFilter(); ws.SetInputData(pd)
ws.SetNumberOfIterations(20); ws.SetPassBand(0.1); ws.Update()
r = ws.GetOutput()
save("filter_windowed_sinc", {"num_points": r.GetNumberOfPoints(), "num_cells": r.GetNumberOfCells(), "bounds": bounds(r)})

# --- Validate ---
print("validate...")
pd = tri_sphere(32)
# Just store basic info - validation is about checking for errors
save("filter_validate", {"num_points": pd.GetNumberOfPoints(), "num_cells": pd.GetNumberOfCells()})

# --- Texture Map Sphere ---
print("texture_map_sphere...")
pd = sphere(32)
tm = vtk.vtkTextureMapToSphere(); tm.SetInputData(pd); tm.Update()
r = tm.GetOutput()
tc = r.GetPointData().GetTCoords()
save("filter_texture_map_sphere", {"num_points": r.GetNumberOfPoints(), "has_tcoords": tc is not None,
     "tcoord_components": tc.GetNumberOfComponents() if tc else 0})

# --- Connectivity Large ---
print("connectivity_large...")
s1 = sphere(128)
s2 = vtk.vtkSphereSource(); s2.SetCenter(3,0,0); s2.SetThetaResolution(128); s2.SetPhiResolution(128); s2.Update()
app = vtk.vtkAppendPolyData(); app.AddInputData(s1); app.AddInputData(s2.GetOutput()); app.Update()
conn = vtk.vtkPolyDataConnectivityFilter(); conn.SetInputData(app.GetOutput())
conn.SetExtractionModeToAllRegions(); conn.ColorRegionsOn(); conn.Update()
r = conn.GetOutput()
save("filter_connectivity_large", {"num_points": r.GetNumberOfPoints(), "num_regions": conn.GetNumberOfExtractedRegions()})

# --- Hausdorff ---
print("hausdorff...")
s1 = sphere(32)
s2 = vtk.vtkSphereSource(); s2.SetCenter(0.05,0,0); s2.SetThetaResolution(32); s2.SetPhiResolution(32); s2.Update()
hd = vtk.vtkHausdorffDistancePointSetFilter()
hd.SetInputData(0, s1); hd.SetInputData(1, s2.GetOutput()); hd.Update()
d = hd.GetHausdorffDistance()
save("filter_hausdorff", {"hausdorff_distance": d})

print("Done generating reference data.")
