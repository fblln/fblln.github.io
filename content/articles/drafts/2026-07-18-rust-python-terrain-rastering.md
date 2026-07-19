+++
title = "What 6.7× Actually Measures: A Rust vs Python Terrain Baker, With GDAL Under Both"
date = "2026-07-18"
description = "Ridgeline's asset baker got a Rust rewrite and a 7× warm-cache speedup. But both bakers call the same GDAL for the heavy geospatial math — so the number isn't what it looks like. A benchmark that measures runtime overhead, not raster kernels."
tags = ["Rust", "Python", "Geospatial", "GDAL", "Performance", "Benchmarks"]
+++

Ridgeline turns a GPX route into an interactive terrain artifact: fetch elevation
and topo data for the route's bounding box, warp it onto a regular grid, bake
relief and slope textures, serialize a heightfield the browser can render. The
baker existed first in Python (`tools/asset-baker/`, ~900 lines of NumPy /
rasterio / Pillow). It was rewritten in Rust (`tools/asset-baker-rs/`). Warm-cache
bakes got roughly **7× faster**.

![Ridgeline terrain artifact — baked relief, slope, and hypsometric textures for the Morning Hike route](/assets/ridgeline.webp)

*The baker's output: a route's terrain baked into relief, slope, and hypsometric layers the browser renders interactively.*

That number is easy to misread, and the interesting part is *how* it's easy to
misread. The obvious story — "Rust replaced slow Python raster loops with a fast
compiled kernel" — is wrong here. **Both bakers call the same GDAL** for the part
that looks expensive: reading the DEM, transforming coordinates, and cubic-
resampling elevation onto the target grid. That native C core is byte-for-byte
identical across the two implementations, by design.

So this benchmark accidentally became a clean experiment. Hold the heavy
geospatial numerics constant — literally the same shared library, the same PROJ,
the same `GDALReprojectImage` call with the same transformer — and measure only
what the *host runtime* costs around it. The 7× is the price of the glue.

## The workload, and the one number people quote

The headline metric on the project card is **6.7×**, a warm-cache compute
speedup. Here is the measurement it comes from. Route: `Morning_Hike.gpx` —
26.72 km, +2056 m, a 1257×1257 elevation grid (~5 m/cell) over a 5.9×6.3 km
extent, with a 7025×7462 px OpenTopoMap texture at zoom 17. Piemonte, Italy —
which matters later. Rust built `--release` with `lto = true` and
`codegen-units = 1`; median of 3 runs each, measured with `/usr/bin/time -l`.

| Implementation | Phase | Wall | User CPU | CPU% | Peak RSS |
| -------------- | ----- | ----:| --------:| ----:| --------:|
| **Rust** (rayon on) | cold | 21.4 s | 3.04 s | 21% | 788 MiB |
| | warm | **2.26 s** | 2.63 s | **124%** | 781 MiB |
| **Rust** (rayon off) | warm | 2.72 s | 2.54 s | 100% | 783 MiB |
| **Python** (same DAG) | cold | 33.5 s | 20.5 s | 67% | 1679 MiB |
| | warm | **16.9 s** | 17.9 s | 111% | 1665 MiB |

The "6.7×" is a *warm-cache compute* ratio: Python burns **17.9 s** of user CPU
to Rust's **2.63 s** (≈6.8×) for identical output, so warm wall-clock is 16.9 s
vs 2.26 s (7.5×). It is **not** an end-to-end speedup, and it is **not** a raster-
kernel speedup. Everything downstream depends on separating those.

## Both roads go through GDAL

DEM sampling is a reprojection: two source rasters in different CRSs (IGN in
EPSG:4326, Piemonte DTM in EPSG:32632) resampled with cubic interpolation onto
the node-aligned target grid. The Python baker does it with rasterio (GDAL's
Python binding):

```python
def _warp_cubic(src, src_transform, src_crs, dst_transform, nx, ny):
    dst = np.full((ny, nx), np.nan, dtype=np.float64)
    reproject(
        source=np.ascontiguousarray(src), destination=dst,
        src_transform=src_transform, src_crs=src_crs, src_nodata=np.nan,
        dst_transform=dst_transform, dst_crs=CRS.from_epsg(4326), dst_nodata=np.nan,
        resampling=Resampling.cubic,
        tolerance=0.0,  # exact transformer — no approximation grid
    )
    return dst
```

The Rust baker does the *same GDAL call* through the `gdal` crate. Not a
reimplementation — the same `libgdal`:

```rust
// DEM sampling is a GDAL cubic reproject onto the target grid, identical to the
// Python baker's rasterio.reproject(resampling=cubic, tolerance=0) — both call
// GDALReprojectImage with the exact transformer, so they match bit-for-bit.
let ign_ds = mem_dataset_4326(&source.ign.data, source.ign.width, source.ign.height, ign_gt)?;
let ign_warp = warp_cubic_4326(&ign_ds, dst_gt, width, height)?;

let mut values = if let Some(piemonte) = &source.piemonte {
    let pie_ds = Dataset::open(&piemonte.path)?;   // GDAL reads the GeoTIFF directly
    let pie_warp = warp_cubic_4326(&pie_ds, dst_gt, width, height)?;
    pie_warp.iter().zip(&ign_warp)                 // Piemonte primary, IGN fills holes
        .map(|(&p, &i)| if p.is_finite() { p } else { i })
        .collect()
} else { ign_warp };
```

Even the coordinate transform is shared. Python uses `pyproj`; Rust uses GDAL's
`CoordTransform` — but pyproj and GDAL bind the *same PROJ*, so the 4326→32632
conversion is byte-identical. The Rust side just caches the transformer per
thread, because building it isn't free and `CoordTransform` isn't `Sync`:

```rust
thread_local! {
    static UTM32: CoordTransform = {
        let mut src = SpatialRef::from_epsg(4326).unwrap();
        let mut dst = SpatialRef::from_epsg(32632).unwrap();
        // GDAL 3 honours CRS axis order; force lon/lat, easting/northing.
        src.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);
        dst.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);
        CoordTransform::new(&src, &dst).unwrap()   // same PROJ pyproj uses
    };
}
```

This was a deliberate reversal. The migration plan originally specified *avoiding*
GDAL in Rust — pure-Rust `tiff` + `proj4rs`, no system dependency. That was
dropped for one reason: **fidelity**. GDAL's cubic reproject prefilters and
edge-handles in ways a hand-rolled bicubic won't reproduce, and the fidelity gate
was ≤0.5 m of height error against the Python golden output. The cheapest way to
hit "bit-for-bit" was to run the identical library. So the Rust baker took on a
GDAL 3.4–3.12 system dependency on purpose — the opposite of the usual "rewrite
it in Rust to drop the C dependency" story.

The payoff for *this* article: the DEM warp, the CRS math, and the GeoTIFF decode
are removed as variables. They cost the same in both. Whatever the 7× is, it
isn't there.

## Cold vs warm: the number depends entirely on which you mean

Split the two phases and the "7×" mostly evaporates — in the direction that
matters to a user.

**Cold** (empty cache — the real first-import experience) is network-bound. Both
Rust configs sit at **21–22% CPU** across a ~21 s wall: only ~4.4 s of actual
compute, the rest waiting on sockets. The gap that survives is the *network*
portion, and it's nearly equal between languages — Python's `cold − warm` ≈ 16.6 s,
Rust's ≈ 19.1 s. A cold bake is dominated by five government DEM/tile servers, and
no amount of Rust makes `data.geopf.fr` answer faster.

**Warm** (populated cache) is the only phase that is pure compute, and it's the
only place the 7× lives. Strip the network and you're timing exactly the host-
runtime overhead this whole piece is about.

The honest reading: the metric is real, but it describes the *second* time you
bake a route, not the first. The end-to-end experience a new user actually has is
network-bound and roughly language-independent.

## Where the warm CPU actually goes

If not the GDAL warp, then what is Python spending 17.9 s on? Profiling the warm
path puts the CPU in four places, and the instinctive one is last:

1. **PNG compression (zlib).** The topo texture is 7025×7462; relief, slope,
   hypsometric, normal, and multishade layers are each grid-sized. Deflating all
   of that is the single biggest CPU line in a warm bake.
2. **Serializing `terrain.json`.** The heightfield ships as JSON text — up to ~9M
   floats, each `round()`ed and formatted to a string. Float-to-decimal at that
   volume is not cheap in any language, and it's a lot cheaper in Rust's `ryu`
   path than in Python's.
3. **The texture enhance / mean passes.** Saturation/brightness/contrast grading
   over the 7025×7462 texture, and the multishade mean over seven light azimuths.
   This is the *only* stage that parallelizes cleanly (see below).
4. **The relief/slope kernels** — hillshade, slope, hypsometric tint, normals.
   The per-pixel math everyone's instinct says to optimize (and to move to the
   GPU). On 9M cells it's **tens of milliseconds**, and it was already native C in
   the Python (NumPy) baker. It is the cheapest stage in the pipeline.

That last point is the trap. The stage that *looks* like the hot loop — shade
every pixel from its neighbors — is a rounding error, and it's the only GPU-
amenable one. The expensive work is compression and serialization: bandwidth- and
allocation-bound bookkeeping, not floating-point. Reaching for `wgpu` here would
optimize the one thing that's already free.

## So why is Python ~7× the CPU and ~2× the memory?

Not the reasons you'd guess first.

**Not the GIL.** Running the baker under free-threaded Python 3.14t
(`Py_GIL_DISABLED=1`) did not improve it. Almost all the expensive work is already
inside native extensions — NumPy, SciPy, rasterio, Pillow, GDAL — so there's no
Python bytecode contending for the lock to release. Warm Python runs at 111% CPU:
barely more than one core, and not because a lock is stopping it.

**Not thread count.** A 4-worker profile recovered almost all the wall-clock of a
16–20 worker fan-out. More threads was never the lever.

**It's the shape of the code.** Both implementations are idiomatic. NumPy
expresses the pipeline as vectorized whole-array operations — concise, readable,
and paid for in *large temporary arrays*: every intermediate materializes a full
grid, and the allocator churns hundreds of megabytes that exist for one line. Rust
expresses the same pipeline as explicit buffers and direct loops with tight
allocation lifetimes — a value is computed, used, and dropped without ever
becoming a resident array. That's the whole 2× memory gap (1.67 GiB vs 0.78 GiB),
and a large part of the compute gap: Python spends real time allocating,
populating, and freeing arrays that Rust never creates.

One measurement makes this concrete. A memory-hunt on the Python side replaced a
per-pixel forest-shading computation with a 101-entry `float32` lookup table
indexed by the `uint8` canopy raster, applied in place — still idiomatic NumPy,
no Rust-style rewrite. Peak RSS dropped from ~1.0 GiB to ~710 MiB, output byte-
identical. The memory wasn't the *problem being solved*; it was temporaries. Rust
just doesn't create them in the first place.

## The `rayon` flag, honestly

Rust's warm advantage over rayon-off Rust is small and worth being precise about.
`WEB_COMPUTE_THREADS` sizes the rayon pool that drives the texture enhance/mean
passes. Turning it on takes warm wall from 2.72 s → 2.26 s: same total user CPU
(2.54 s vs 2.63 s), just spread across cores — 100% CPU → 124%. That's ~0.46 s,
about **18%**, and *only* on the texture passes. The GDAL warps, the serial relief
loops, and single-threaded PNG encode don't move. Output is byte-identical with
rayon on or off (same texture hash). Parallelism helped exactly the stage that was
parallelizable, and no more — which is the honest ceiling, not a disappointment.

## What actually made bakes fast: the cold path

The 7× is a compute story, but the *engineering* that improved real bakes lives on
the cold, network-bound path — and it helped **both** bakers, because the wins are
architectural, not language-level. A cold bake pulls five independent sources
(IGN elevation, Piemonte DTM, OpenTopoMap tiles, Copernicus forest, an Overpass
border query), each keyed only on the route bbox. Three changes mattered:

**Pooled HTTP.** A single process-wide agent reuses connections per host instead
of a fresh client per request; in-memory reads carry a body limit, cache writes
stream to a temp file with atomic rename.

**A coverage probe before a 150 MB mistake.** IGN HD only covers France. An
Italian track would download the full grid — here ~150 MB of BIL float32 — purely
to discard it, then cubic-warp 38 million nodata cells. One 64×64 WMS probe
settles coverage first:

```rust
// One cheap 64x64 probe settles coverage; when dry, a tiny all-NaN grid is
// semantically identical to the old full all-NaN grid (Piemonte fills it).
if !ign_has_coverage(bounds)? {
    let dem = Grid::new(IGN_PROBE_RES, IGN_PROBE_RES, f64::NAN);
    cache::write_grid(&cache, &dem)?;
    return Ok(dem);
}
```

The effect, on this Italian route:

| | before probe | after probe |
| ---------------- | ------------:| -----------:|
| IGN cache size | 146 MB | **16 KB** |
| total cold cache | 167.7 MB | 22.2 MB |
| peak RSS | 1.76 GiB | 791 MiB |
| texture output | `7517ecf7…` | `7517ecf7…` (identical) |

And it stays correct where it should fetch: Chamonix (France) probes 4096/4096
cells and downloads; a Bardonecchia track straddling the border probes 4092/4096
and downloads; this interior-Italian track probes 0/4096 and skips. Mirrored in
both bakers.

**An all-sources DAG.** Fire all five downloads at once and gate each compute step
on the join of *exactly* its inputs — DEM build waits on IGN + Piemonte and starts
immediately, without blocking on the topo/forest/border fetches. Because the
authoritative stages re-fetch on a cache miss, the overlapped run is byte-identical
to a sequential one; only the I/O overlaps. Cold wall drops from ~40 s sequential
to ~21 s — the floor being the single slowest server. Beating *that* needs a
faster source (a Copernicus GLO-30 COG over `/vsicurl/`, say), not more threads
and not a different language.

## The takeaway

Measure the real system, and label the number honestly.

- **6.7× is a warm-cache compute ratio.** It's real, it's reproducible, and it
  describes re-baking a cached route — not a user's first import, which is
  network-bound and roughly language-independent.
- **The geospatial core is identical.** Both bakers call the same GDAL and the
  same PROJ, bit-for-bit, on purpose. The speedup is entirely in the host runtime
  around that core: fewer temporary arrays, tighter allocation lifetimes, faster
  float serialization, half the memory.
- **The obvious optimization was the wrong one.** The per-pixel relief kernel —
  the thing that looks like the hot loop and the thing you'd move to the GPU — is
  the cheapest stage and was already native C. The CPU was in zlib and
  float-to-string.
- **The biggest wins weren't the rewrite.** Not refetching (cache), not
  downloading-then-discarding (coverage probe), and overlapping I/O (DAG) helped
  both implementations more than the language switch helped either.

> A benchmark is only as honest as its label. "6.7× faster" is true; "6.7× faster
> at re-baking a cached route, in host-runtime overhead around an unchanged GDAL
> core, with no change to the network-bound first run" is *useful*. The second one
> tells you where not to spend your next week.
