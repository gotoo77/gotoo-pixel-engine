# GPE.UI MFE-001C — Windows Cost Baseline

Status: **OBSERVATION — NOT A FINAL COST VERDICT**

Environment reported by the human runtime gate:

```text
OS: Windows
arch: x86_64
build: cargo run --release --example gpe_ui_mfe_001c_cost_probe
warmup iterations: 250
sample iterations: 2000
```

## Observed results

```text
TINY
logical_items=4
persistent_logical_entries=1
allocation_calls median=28 p95=28
allocation_bytes median=2076 p95=2076
transaction_ns min=3100 median=3500 p95=3700 max=23400

SPATIAL_GRID_6
logical_items=6
persistent_logical_entries=1
allocation_calls median=11 p95=11
allocation_bytes median=1315 p95=1315
transaction_ns min=1900 median=2300 p95=4800 max=32300

TYPE SIZES
UiStateStore=96 bytes
UiOutput=232 bytes
SpatialState=104 bytes
SpatialOutput=168 bytes

CURRENT PROBE EXECUTABLE
236544 bytes
```

## Interpretation boundary

The transaction timings are observationally very small on this machine. No threshold is invented from these numbers.

The allocation counts are **not yet attributable to the minimal UI kernel alone**. The current experimental paths construct textual headless/debug dumps on every transaction, and those allocations are included by the counting allocator.

Therefore:

```text
CPU COST SIGNAL:
NO CURRENT RED FLAG AT THIS SCALE

ALLOCATION COST SIGNAL:
REQUIRES ATTRIBUTION BEFORE ARCHITECTURAL VERDICT

236544-BYTE EXECUTABLE:
ABSOLUTE OBSERVATION ONLY
NOT AN INCREMENTAL GPE.UI SIZE DELTA
```

MFE-001C continues with bounded same-toolchain control / Tiny / Spatial artifact comparisons for Native and WASM.
