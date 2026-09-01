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

## Observed transaction results

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

## Observed artifact sizes

Same branch / release profile / pinned toolchain, measured on the Windows human environment.

### Native x86_64 Windows executables

```text
control std-only   115200 bytes
tiny UI            169472 bytes
spatial Grid       187392 bytes

raw deltas:
tiny - control      54272 bytes
spatial - control   72192 bytes
spatial - tiny      17920 bytes
```

### Raw wasm32-unknown-unknown modules

```text
control std-only      22538 bytes
tiny UI             1444176 bytes
spatial Grid        1462432 bytes

raw deltas:
tiny - control      1421638 bytes
spatial - control   1439894 bytes
spatial - tiny        18256 bytes
```

The duplicated hashed/non-hashed WASM files produced by Cargo had identical sizes; each target is counted once above.

## Interpretation boundary

The transaction timings are observationally very small on this machine. No threshold is invented from these numbers.

The allocation counts are **not yet attributable to the minimal UI kernel alone**. The current experimental paths construct textual headless/debug dumps on every transaction, and those allocations are included by the counting allocator.

The std-only artifact control does **not reference the GPE crate at all**. Therefore the very large raw WASM `tiny - control` delta must not be described as "GPE.UI adds ~1.4 MB". It conflates activation/linkage of GPE code and transitive code with the UI path itself.

The most internally comparable observation currently available is the additional Spatial path over Tiny:

```text
Native spatial - tiny = 17920 bytes
Raw WASM spatial - tiny = 18256 bytes
```

Those are observations, not acceptance thresholds.

Therefore:

```text
CPU COST SIGNAL:
NO CURRENT RED FLAG AT THIS SCALE

ALLOCATION COST SIGNAL:
REQUIRES ATTRIBUTION / PRODUCTIONIZATION REVIEW
NO CURRENT ARCHITECTURE-B REJECTION SIGNAL

STD-ONLY -> UI ARTIFACT DELTA:
NOT A PURE UI DELTA
REQUIRES A GPE-NON-UI CONTROL

SPATIAL -> TINY INCREMENT:
BOUNDED AND OBSERVED
NO ACCEPTABILITY THRESHOLD INVENTED
```

MFE-001C continues with a GPE non-UI artifact control, Native runtime confirmation, and the Web browser runtime gate when an actual WebGPU-capable environment is available.
