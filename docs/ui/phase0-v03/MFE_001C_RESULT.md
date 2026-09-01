# GPE.UI MFE-001C — RESULT

Status: **PASS WITH CONDITIONS**

Architecture checkpoint: **ARCHITECTURE B = GO**

This result closes the experimental cost/runtime gate that follows MFE-001A and MFE-001B.

## Scope exercised

MFE-001C measured and validated:

- headless Tiny transaction timing and allocations;
- headless six-card Spatial Grid timing and allocations;
- inline state/output type sizes;
- bounded Native artifact sizes;
- bounded raw WASM artifact sizes;
- a meaningful GPE-core/non-UI artifact control;
- Native runtime through the previously accepted MFE probes plus the cost probe;
- CI Native validation;
- CI Web build/package validation;
- an actual Chrome/WebGPU browser runtime of the MFE-001B spatial probe.

No production optimization, styling system, typography change, markup, SVG, Taffy integration, Arcade migration, legacy-UI removal, or public API freeze was performed.

## Transaction-cost observations

Windows x86_64, release mode, 250 warmup iterations and 2000 measured iterations:

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
```

Observed CPU cost presents **no current architecture-rejection signal at this scale**.

The allocation counts include construction of current experimental textual/headless diagnostic dumps. They are therefore not yet attributable to the minimal production kernel alone.

This remains a productionization follow-up; it is not evidence requiring Architecture B rejection.

## State/output size observations

```text
UiStateStore   96 bytes
UiOutput      232 bytes
SpatialState  104 bytes
SpatialOutput 168 bytes
```

These are inline Rust type sizes only; heap capacities are excluded.

## Artifact observations

The meaningful control references GPE `Framebuffer` + `Pixel` but no experimental UI.

### Native Windows x86_64

```text
std-only control   115200 bytes
GPE core           117248 bytes
Tiny UI            169472 bytes
Spatial Grid       187392 bytes

Tiny - GPE core      52224 bytes
Spatial - GPE core   70144 bytes
Spatial - Tiny       17920 bytes
```

### Raw wasm32-unknown-unknown

```text
std-only control       22538 bytes
GPE core             1354957 bytes
Tiny UI              1444176 bytes
Spatial Grid         1462432 bytes

Tiny - GPE core        89219 bytes
Spatial - GPE core    107475 bytes
Spatial - Tiny         18256 bytes
```

For this bounded WASM comparison:

```text
Tiny over GPE core       +6.58%
Spatial over GPE core    +7.93%
Spatial over Tiny        +1.26%
```

The earlier apparent ~1.4 MB `std-only -> UI` jump is explicitly rejected as a UI-cost interpretation: most of that size is already present in the GPE-core control before the experimental UI path is referenced.

No product acceptance threshold is invented from these numbers.

## Runtime gates

### Native

**PASS**

Native behavior has been exercised through MFE-001A, MFE-001B, and the MFE-001C release cost probe. The accepted 001B runtime covered responsive layout, spatial focus, dynamic filtering/reordering, custom Card rendering and multimodal interaction.

### Web browser

**PASS WITH PLATFORM-LIFECYCLE NOTE**

Actual runtime was exercised in Chrome on Windows through a local HTTP server and the MFE-001B Web/WASM probe.

Observed working behavior included live rendering and interaction, including responsive width switching, filtering/reordering and spatial focus/card behavior. The browser runtime was not merely inferred from a successful build/package.

One separate generic GPE lifecycle observation was found:

```text
Escape / gamepad East
    -> probe returns GameResult::Exit
    -> Web runtime calls event_loop.exit()
    -> event loop stops
    -> browser retains the last canvas frame
    -> page appears frozen until reload
```

This is not classified as a GPE.UI failure. It is a Web embedding/lifecycle contract question and is tracked separately as:

`#73 — web: define GameResult::Exit browser lifecycle semantics`

MFE-001C does not change generic engine exit semantics.

## CI gate

The pre-result MFE-001C head passed:

```text
Native                  PASS
Web build/package       PASS
Conventional Commits    PASS
```

The final result commit must also remain green before merge.

## Conditions carried into productionization

1. **Allocation attribution**
   - separate mandatory production work from diagnostic/headless dump overhead before treating current allocation counts as production-kernel cost;
   - optimize only with evidence; do not invent a zero-allocation requirement.

2. **Web exit lifecycle**
   - resolve/document issue #73 independently of GPE.UI;
   - do not special-case UI to hide generic runtime lifecycle semantics.

Neither condition blocks Architecture B productionization planning.

## Architecture B checkpoint

```text
MFE-001A  PASS
MFE-001B  PASS
MFE-001C  PASS WITH CONDITIONS

ARCHITECTURE B = GO
```

Rationale:

- the transaction model survived implementation and human runtime testing;
- typed proposals and identity/state semantics survived the 001A gate;
- responsive integer layout, dynamic keyed collections, spatial navigation and custom painting survived the 001B gate;
- Native and actual Web browser runtime were demonstrated;
- measured CPU and bounded artifact costs show no architecture-rejection signal;
- the remaining allocation and Web-exit questions are bounded production/lifecycle concerns rather than evidence against the architecture.

## What GO authorizes

`GO` authorizes **productionization planning** for Architecture B.

It does **not** authorize:

- immediate public API freeze;
- removal of the legacy UI path;
- automatic Arcade migration;
- premature crate extraction;
- markup/SVG/Taffy expansion;
- ignoring the styling/theming and typography work still required for a useful GPE.UI v1.

The next phase should convert the proven experimental model into a deliberately shaped production API, then validate it with real consumers.