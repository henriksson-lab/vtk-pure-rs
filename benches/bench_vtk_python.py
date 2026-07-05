"""Benchmark VTK C++ (via Python bindings) for comparison with vtk-rs."""
import resource
import time
import vtk


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


def format_kib(value):
    if value is None:
        return "n/a"
    if value >= 1024:
        return f"{value / 1024:.1f} MiB"
    return f"{value} KiB"


def format_peak_delta(before, after):
    if before is None or after is None:
        return "n/a"
    if after >= before:
        return f"+{format_kib(after - before)}"
    return format_kib(after)


def make_sphere(resolution):
    src = vtk.vtkSphereSource()
    src.SetThetaResolution(resolution)
    src.SetPhiResolution(resolution)
    src.Update()
    return src.GetOutput()


def make_image_data(size):
    img = vtk.vtkImageData()
    img.SetDimensions(size, size, size)
    sp = 1.0 / size
    img.SetSpacing(sp, sp, sp)
    scalars = vtk.vtkDoubleArray()
    scalars.SetNumberOfTuples(size * size * size)
    scalars.SetName("scalars")
    idx = 0
    for k in range(size):
        for j in range(size):
            for i in range(size):
                x = i / size - 0.5
                y = j / size - 0.5
                z = k / size - 0.5
                scalars.SetValue(idx, (x*x + y*y + z*z)**0.5 - 0.3)
                idx += 1
    img.GetPointData().SetScalars(scalars)
    return img


def bench(name, iterations, func):
    rss_before = current_rss_kib()
    peak_before = peak_rss_kib()

    # Warm up
    func()

    start = time.perf_counter()
    for _ in range(iterations):
        func()
    elapsed = time.perf_counter() - start
    per_iter = elapsed / iterations

    if per_iter >= 0.001:
        per_str = f"{per_iter * 1000:.2f} ms"
    else:
        per_str = f"{per_iter * 1_000_000:.1f} us"

    rss_after = current_rss_kib()
    peak_after = peak_rss_kib()
    print(
        f"  {name:<35} {per_str:>10}  "
        f"rss {format_kib(rss_after):>9}  "
        f"peak {format_peak_delta(peak_before, peak_after):>9} "
        f"({iterations} iters)"
    )


def bench_sphere_gen(resolution):
    def f():
        src = vtk.vtkSphereSource()
        src.SetThetaResolution(resolution)
        src.SetPhiResolution(resolution)
        src.Update()
        return src.GetOutput()
    return f


def bench_normals(pd):
    def f():
        n = vtk.vtkPolyDataNormals()
        n.SetInputData(pd)
        n.ComputePointNormalsOn()
        n.ComputeCellNormalsOn()
        n.Update()
        return n.GetOutput()
    return f


def bench_elevation(pd):
    def f():
        e = vtk.vtkElevationFilter()
        e.SetInputData(pd)
        e.SetLowPoint(0, 0, -1)
        e.SetHighPoint(0, 0, 1)
        e.Update()
        return e.GetOutput()
    return f


def bench_marching_cubes(img):
    def f():
        mc = vtk.vtkMarchingCubes()
        mc.SetInputData(img)
        mc.SetValue(0, 0.0)
        mc.Update()
        return mc.GetOutput()
    return f


def bench_triangulate(pd):
    def f():
        t = vtk.vtkTriangleFilter()
        t.SetInputData(pd)
        t.Update()
        return t.GetOutput()
    return f


def bench_clean(pd):
    def f():
        c = vtk.vtkCleanPolyData()
        c.SetInputData(pd)
        c.Update()
        return c.GetOutput()
    return f


def bench_decimate(pd):
    def f():
        d = vtk.vtkDecimatePro()
        d.SetInputData(pd)
        d.SetTargetReduction(0.5)
        d.Update()
        return d.GetOutput()
    return f


def bench_smooth(pd):
    def f():
        s = vtk.vtkSmoothPolyDataFilter()
        s.SetInputData(pd)
        s.SetNumberOfIterations(20)
        s.SetRelaxationFactor(1.0)
        s.FeatureEdgeSmoothingOn()
        s.Update()
        return s.GetOutput()
    return f


def main():
    print(f"VTK C++ (Python bindings) v{vtk.vtkVersion.GetVTKVersion()} benchmarks")
    print("=" * 50 + "\n")

    bench("sphere(16) generation", 100, bench_sphere_gen(16))
    bench("sphere(64) generation", 10, bench_sphere_gen(64))

    pd16 = make_sphere(16)
    pd32 = make_sphere(32)
    pd64 = make_sphere(64)

    bench("normals (sphere 16)", 100, bench_normals(pd16))
    bench("normals (sphere 64)", 10, bench_normals(pd64))
    bench("elevation (sphere 64)", 20, bench_elevation(pd64))

    img32 = make_image_data(32)
    bench("marching_cubes (32^3)", 10, bench_marching_cubes(img32))
    bench("triangulate (sphere 32)", 50, bench_triangulate(pd32))
    bench("clean (sphere 32)", 20, bench_clean(pd32))
    bench("decimate 50% (sphere 32)", 10, bench_decimate(pd32))
    bench("smooth 20 iters (sphere 32)", 10, bench_smooth(pd32))


if __name__ == "__main__":
    main()
