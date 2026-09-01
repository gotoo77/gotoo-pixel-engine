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

Two controls are retained:

```text
std-only control
    does not reference GPE at all

GPE-core control
    references GPE Framebuffer + Pixel
    does not reference experimental UI
```

The GPE-core control is the meaningful control for estimating the incremental UI path in these bounded binaries.

### Native x86_64 Windows executables

```text
std-only control   115200 bytes
GPE core           117248 bytes
tiny UI            169472 bytes
spatial Grid       187392 bytes

raw deltas:
GPE core - std       2048 bytes
tiny - GPE core     52224 bytes
spatial - GPE core  70144 bytes
spatial - tiny      17920 bytes
```

Relative observations for this specific bounded comparison:

```text
tiny over GPE core      +44.54%
spatial over GPE core   +59.83%
spatial over tiny       +10.57%
```

These percentages are properties of very small executables and must not be generalized to full games.

### Raw wasm32-unknown-unknown modules

```text
std-only control       22538 bytes
GPE core             1354957 bytes
tiny UI              1444176 bytes
spatial Grid         1462432 bytes

raw deltas:
GPE core - std       1332419 bytes
tiny - GPE core        89219 bytes
spatial - GPE core    107475 bytes
spatial - tiny         18256 bytes
```

Relative observations for this specific bounded comparison:

```text
tiny over GPE core       +6.58%
spatial over GPE core    +7.93%
spatial over tiny        +1.26%
```

The duplicated hashed/non-hashed WASM files produced by Cargo had identical sizes; each target is counted once above.

## Interpretation boundary

The transaction timings are observationally very small on this machine. No threshold is invented from these numbers.

The allocation counts are **not yet attributable to the minimal UI kernel alone**. The current experimental paths construct textual headless/debug dumps on every transaction, and those allocations are included by the counting allocator.

The original std-only comparison demonstrated why a same-crate control was necessary: most of the ~1.4 MB raw-WASM jump occurs before the experimental UI path is referenced at all.

With the GPE-core control, the bounded raw-WASM incremental observations are:

```text
GPE core -> Tiny UI       +89219 bytes
GPE core -> Spatial Grid +107475 bytes
Tiny UI  -> Spatial Grid  +18256 bytes
```

Therefore it is incorrect to state that GPE.UI itself adds ~1.4 MB WASM in this experiment.

The Native comparison has larger relative percentages because the bounded control executable is very small; the absolute observed increments remain 52–70 KB. No production-game percentage is inferred from these probes.

Therefore:

```text
CPU COST SIGNAL:
NO CURRENT RED FLAG AT THIS SCALE

ALLOCATION COST SIGNAL:
REQUIRES ATTRIBUTION / PRODUCTIONIZATION REVIEW
NO CURRENT ARCHITECTURE-B REJECTION SIGNAL

ARTIFACT COST SIGNAL:
BOUNDED INCREMENT NOW ATTRIBUTED AGAINST A GPE NON-UI CONTROL
NO CURRENT ARCHITECTURE-B REJECTION SIGNAL
NO ACCEPTABILITY THRESHOLD INVENTED

RAW WASM ~1.4 MB CLAIM:
REJECTED AS AN INTERPRETATION
MOST OF THAT SIZE IS ALREADY PRESENT IN THE GPE-CORE CONTROL
```

## Remaining MFE-001C gates

```text
Native functional/runtime gate       observed through MFE-001A, MFE-001B and cost probe
CI Native                            required green on final head
CI Web build/package                 required green on final head
Actual browser runtime               required only when a WebGPU-capable environment is available
Allocation attribution               may remain a bounded productionization follow-up if no rejection signal emerges
Architecture B checkpoint            GO / REVISE / REJECT
```

MFE-001C now continues with the final runtime/CI checkpoint and Architecture B decision.
