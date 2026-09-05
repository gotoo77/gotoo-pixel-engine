# GPE.UI — P5 Cost Attribution / Debug Boundary v0.1

Status: **MEASUREMENT PASS — FINAL HYGIENE / WASM GATE PENDING**

## Why P5 exists

MFE-001C found no CPU red flag, but its allocation measurements could not be read as
production-kernel cost because the measured transactions always constructed the
deterministic textual debug dump.

The converged transactional path historically materialized that dump in
`UiOutput`, and the Spatial compatibility adapter similarly materialized a dump
in `SpatialOutput`. These strings are valuable for deterministic headless tests
and diagnostics, but they are not mandatory semantic runtime output.

P5 therefore measures before optimizing.

## Historical baseline evidence from MFE-001C

Release-mode Windows x86_64, 250 warmup iterations and 2,000 measured iterations:

```text
TINY
logical_items=4
persistent_logical_entries=1
allocation_calls median=28 p95=28
allocation_bytes median=2076 p95=2076
transaction_ns median≈3500 p95≈3700

SPATIAL_GRID_6
logical_items=6
persistent_logical_entries=1
allocation_calls median=11 p95=11
allocation_bytes median=1315 p95=1315
transaction_ns median≈2300 p95≈4800
```

Those allocation figures include debug-dump construction and are therefore
**observational totals, not production-kernel attribution**.

## P5 pre-boundary comparison anchor — 2026-09-05

A fresh release-mode run was taken on the current P0–P4 branch immediately before
introducing the debug-capture boundary. Environment: Windows x86_64, 250 warmup
iterations and 2,000 measured iterations.

```text
TINY
logical_items=4
persistent_logical_entries=1
allocation_calls median=35 p95=35
allocation_bytes median=2648 p95=2648
transaction_ns min=3700 median=4200 p95=6100 max=85400

SPATIAL_GRID_6
logical_items=6
persistent_logical_entries=1
allocation_calls median=12 p95=12
allocation_bytes median=1411 p95=1411
transaction_ns min=2000 median=2400 p95=5600 max=87300

TYPE SIZES
UiStateStore=160 bytes
UiOutput=264 bytes
SpatialState=104 bytes
SpatialOutput=176 bytes

CURRENT PROBE EXECUTABLE
bytes=285184
```

This fresh run is the direct P5 comparison anchor. It was still taken with the
old unconditional dump behavior, so the allocation totals still mixed mandatory
runtime work and diagnostic serialization.

## Implemented boundary

The P5 code slice introduces an explicit persistent debug-capture switch on
`UiStateStore` and `SpatialState`.

Default state is capture OFF. `set_debug_capture(true)` opts a state instance into
deterministic textual serialization. Both transactional and Spatial paths still
use the same interaction/layout runtime; only the final textual serialization is
conditional.

Tests cover both sides of the boundary:

```text
capture OFF -> dump is empty
capture ON  -> deterministic dump is available
semantic focus/layout/metrics remain equivalent
painted runtime paths remain capture OFF by default
```

The cost probe runs four separately warmed/measured cases using identical logical
workloads and separate persistent state instances.

## Reportable P5 OFF/ON measurement — 2026-09-05

Environment: Windows x86_64, release profile, 250 warmup iterations, 2,000 measured
iterations.

```text
TINY_CAPTURE_OFF
logical_items=4
persistent_logical_entries=1
allocation_calls median=17 p95=17
allocation_bytes median=1676 p95=1676
transaction_ns min=1400 median=1700 p95=3600 max=13800

TINY_CAPTURE_ON
logical_items=4
persistent_logical_entries=1
allocation_calls median=35 p95=35
allocation_bytes median=2648 p95=2648
transaction_ns min=3700 median=3900 p95=5900 max=23900

SPATIAL_GRID_6_CAPTURE_OFF
logical_items=6
persistent_logical_entries=1
allocation_calls median=6 p95=6
allocation_bytes median=592 p95=592
transaction_ns min=600 median=700 p95=1200 max=30200

SPATIAL_GRID_6_CAPTURE_ON
logical_items=6
persistent_logical_entries=1
allocation_calls median=12 p95=12
allocation_bytes median=1411 p95=1411
transaction_ns min=2000 median=2200 p95=5300 max=25500

TYPE SIZES
UiStateStore=168 bytes
UiOutput=264 bytes
SpatialState=112 bytes
SpatialOutput=176 bytes

CURRENT PROBE EXECUTABLE
bytes=294400
```

## Attribution result

The hypothesis is confirmed: deterministic textual serialization was a material
part of the measured MFE-001C transaction cost.

For the tiny transactional workload, capture OFF versus ON removes:

```text
18 allocation calls / transaction   (35 -> 17, about -51%)
972 requested allocation bytes       (2648 -> 1676, about -37%)
2200 ns median transaction time      (3900 -> 1700, about -56%)
```

For the six-card spatial workload, capture OFF versus ON removes:

```text
6 allocation calls / transaction     (12 -> 6, -50%)
819 requested allocation bytes        (1411 -> 592, about -58%)
1500 ns median transaction time       (2200 -> 700, about -68%)
```

The capture-ON allocation counts and bytes exactly reproduce the fresh
pre-boundary anchor for both workloads. Median timings are also close enough to
support the attribution without claiming cycle-level determinism.

The normal capture-OFF path therefore provides materially clearer production-cost
numbers without introducing a second interaction/layout runtime.

## Secondary size observations

The persistent capture flag increases the inline state sizes observed by this
probe:

```text
UiStateStore   160 -> 168 bytes (+8)
SpatialState   104 -> 112 bytes (+8)
```

`UiOutput` and `SpatialOutput` inline sizes are unchanged. The probe executable
changed from 285184 to 294400 bytes (+9216), but this is **not** treated as an
incremental UI binary-size delta: the executable now contains the expanded
OFF/ON probe and reporting code, so the comparison is confounded by the probe
itself.

No additional allocator/arena/slab optimization is justified by P5 evidence.
The remaining capture-OFF allocations belong to the actual current runtime data
structures and should only be attacked if a future consumer or profiler shows a
real budget problem.

## Compatibility direction

This is still a productionization phase and the public API is not frozen.
Nevertheless, avoid gratuitous test churn.

Current shape:

1. one interaction/layout transaction path;
2. debug capture policy lives on the corresponding persistent UI state;
3. normal default state uses capture OFF;
4. deterministic tests/debug tooling opt in with `set_debug_capture(true)`;
5. cost probes exercise both OFF and ON using otherwise identical workloads;
6. no second interaction/layout runtime is introduced.

The exact final public naming and whether capture policy remains state-owned are
deferred to P7 API cleanup. P5 only needs a falsifiable attribution boundary.

## Validation evidence

Local Native test execution after the P5 boundary:

```text
cargo test
325 library tests passed
2 main tests passed
integration suites passed
36 snake tests passed
doc tests passed
0 failures
```

`cargo fmt --check` did not pass yet, but only reported rustfmt-only diffs in the
P5 probe plus previously touched P4/P5 UI files; no semantic compiler/test failure
was reported. The P5 probe formatting diff has been corrected in-repository.

## Acceptance gates

P5 can PASS only when all of the following are true:

- [x] normal production transaction paths do not construct textual dumps by default;
- [x] debug/headless tests can still obtain deterministic dumps explicitly;
- [x] existing semantic behavior remains covered by tests;
- [x] release-mode cost probe reports capture OFF and ON separately;
- [x] allocation attribution is materially clearer than MFE-001C;
- [x] optimization conclusions follow the new evidence rather than a zero-allocation ideology;
- [ ] repository-wide `cargo fmt --check` is clean;
- [ ] Web/WASM compilation remains valid for the affected UI code.

No human visual runtime gate is required because P5 changes only capture and
measurement boundaries; rendering and input semantics are unchanged.

## Explicit non-goals

P5 does not authorize:

```text
zero-allocation as a requirement
arena/slab introduction without evidence
custom allocator in production
unsafe optimization
new UI architecture
removal of deterministic debug dumps
public API freeze
P6 consumer migration
```

## Current STOP condition

The cost-attribution experiment is complete and reviewed. Do not optimize the
remaining OFF-path allocations in P5.

Finish only the hygiene and Web compilation gates, then mark P5 PASS / STOP and
advance to P6 consumer migration.
