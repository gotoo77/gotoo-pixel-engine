# GPE.UI — P5 Cost Attribution / Debug Boundary v0.1

Status: **ACTIVE — CODE SLICE READY, WAITING FOR REPORTABLE OFF/ON MEASUREMENT**

## Why P5 exists

MFE-001C found no CPU red flag, but its allocation measurements cannot be read as
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
old unconditional dump behavior, so the allocation totals still mix mandatory
runtime work and diagnostic serialization.

## P5 hypothesis

A substantial part of the observed allocation volume belongs to deterministic
diagnostic serialization rather than mandatory interaction/layout/value output.

The hypothesis is falsifiable by running the same workloads with debug capture
OFF and ON under the same allocator, toolchain and release-mode process.

## Required implementation boundary

The transaction must expose the same semantic behavior whether textual debug
capture is disabled or enabled.

Mandatory output includes only data required by runtime semantics, such as:

```text
interaction result
focus / hover / capture state
activations / ActionId output
changed typed values
diagnostics that are part of correctness contracts
metrics that can be represented without textual tree serialization
resolved geometry required by the active adapter/consumer
```

Opt-in debug output includes:

```text
deterministic textual tree dump
deterministic spatial layout dump
human-readable serialization used only by tests/probes/debug tooling
```

P5 must not remove deterministic dumps. It must stop making their construction a
mandatory cost of every transaction.

## Implemented first slice

The current P5 code slice introduces an explicit persistent debug-capture switch
on `UiStateStore` and `SpatialState`.

Default state is capture OFF. `set_debug_capture(true)` opts a state instance into
deterministic textual serialization. Both transactional and Spatial paths still
use the same interaction/layout runtime; only the final textual serialization is
conditional.

Tests now cover both sides of the boundary:

```text
capture OFF -> dump is empty
capture ON  -> deterministic dump is available
semantic focus/layout/metrics remain equivalent
painted runtime paths remain capture OFF by default
```

The cost probe now runs four separately warmed/measured cases using identical
logical workloads and separate persistent state instances.

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

## Measurement matrix

Measure at least:

```text
tiny transaction       capture OFF
tiny transaction       capture ON
spatial grid 6         capture OFF
spatial grid 6         capture ON
```

For each case report:

```text
logical item count
persistent logical entries
allocation calls median / p95
allocation bytes median / p95
transaction ns min / median / p95 / max
```

Also report type sizes and executable/artifact size as separate observations.
Do not mix binary-size observations with per-transaction allocation conclusions.

## Acceptance gates

P5 can PASS only when all of the following are true:

- normal production transaction paths do not construct textual dumps by default;
- debug/headless tests can still obtain deterministic dumps explicitly;
- existing semantic behavior remains covered by tests;
- release-mode cost probe reports capture OFF and ON separately;
- allocation attribution is materially clearer than MFE-001C;
- any proposed optimization is justified by the new evidence rather than by a
  zero-allocation ideology;
- Native/Web compilation remains valid for the affected UI code.

No human visual runtime gate is required if P5 changes only capture/measurement
boundaries and semantic tests prove identical outputs. If rendering or input
semantics change, that assumption is void and runtime validation must be added.

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

The first implementation slice now addresses the two known unconditional dump
producers:

- transactional `UiOutput` in `src/ui/experimental.rs`;
- compatibility `SpatialOutput` in `src/ui/experimental_spatial.rs`.

`examples/gpe_ui_mfe_001c_cost_probe.rs` now measures capture OFF/ON without
changing the logical workloads.

**STOP after the next reportable release-mode measurements.** Review the evidence
before optimizing anything further.
