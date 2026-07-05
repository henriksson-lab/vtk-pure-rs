"""Benchmark VTK C++ (via Python bindings) — larger workloads for fair comparison."""
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

    func()  # warmup
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


def main():
    print(f"VTK C++ (Python) v{vtk.vtkVersion.GetVTKVersion()} LARGE benchmarks")
    print("=" * 55 + "\n")

    pd256 = make_sphere(256)
    print(f"  sphere(256) points: {pd256.GetNumberOfPoints()}")

    def bench_normals():
        n = vtk.vtkPolyDataNormals()
        n.SetInputData(pd256)
        n.ComputePointNormalsOn()
        n.ComputeCellNormalsOn()
        n.Update()
        return n.GetOutput()

    bench("normals (sphere 256)", 5, bench_normals)

    def bench_elevation():
        e = vtk.vtkElevationFilter()
        e.SetInputData(pd256)
        e.SetLowPoint(0, 0, -1)
        e.SetHighPoint(0, 0, 1)
        e.Update()
        return e.GetOutput()

    bench("elevation (sphere 256)", 5, bench_elevation)

    img64 = make_image_data(64)

    def bench_mc():
        mc = vtk.vtkMarchingCubes()
        mc.SetInputData(img64)
        mc.SetValue(0, 0.0)
        mc.Update()
        return mc.GetOutput()

    bench("marching_cubes (64^3)", 5, bench_mc)

    pd128 = make_sphere(128)

    def bench_tri():
        t = vtk.vtkTriangleFilter()
        t.SetInputData(pd128)
        t.Update()
        return t.GetOutput()

    bench("triangulate (sphere 128)", 10, bench_tri)

    def bench_clean():
        c = vtk.vtkCleanPolyData()
        c.SetInputData(pd128)
        c.Update()
        return c.GetOutput()

    bench("clean (sphere 128)", 5, bench_clean)

    def bench_dec():
        d = vtk.vtkDecimatePro()
        d.SetInputData(pd128)
        d.SetTargetReduction(0.5)
        d.Update()
        return d.GetOutput()

    bench("decimate 50% (sphere 128)", 5, bench_dec)

    def bench_smooth():
        s = vtk.vtkSmoothPolyDataFilter()
        s.SetInputData(pd128)
        s.SetNumberOfIterations(20)
        s.SetRelaxationFactor(1.0)
        s.FeatureEdgeSmoothingOn()
        s.Update()
        return s.GetOutput()

    bench("smooth 20 iters (sphere 128)", 5, bench_smooth)


if __name__ == "__main__":
    main()
