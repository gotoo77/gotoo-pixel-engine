# GPE.UI MFE-001C — COST / RUNTIME PLAN

Status: **PLAN — IMPLEMENTATION NOT STARTED**

Baseline:

`f9cf9ce7e4b79f46839425bc38171b289eb922ea`

Prerequisites:

```text
MFE-001A  PASS
MFE-001B  PASS
```

## Mission

Measure the cost and runtime implications of the experimental Architecture B path before productionization.

MFE-001C must measure before optimizing. It must not invent threshold values or broaden into API cleanup, styling, typography, markup, SVG, Arcade migration, or production stabilization.

## Required measurements

### Headless cost probes

Measure separately for a small UI and the responsive spatial Card Grid:

```text
allocations / transaction
allocated bytes / transaction
transaction wall time
persistent state size
node/card count
```

Timing measurements are observational and must be reported with environment/build-mode context. No single run is treated as a benchmark truth.

### Native artifact size

Measure release executable sizes for bounded comparison targets built from the same commit/toolchain.

Report raw byte sizes and deltas only. Do not label a delta acceptable/unacceptable without an explicit product decision.

### Web artifact size

Measure WASM/package output for equivalent bounded targets where the existing GPE packaging path supports it.

A successful WASM build/package is **not** a browser runtime PASS.

### Runtime gates

```text
Native runtime       REQUIRED
Web browser runtime  REQUIRED IF AN ACTUAL WEBGPU-CAPABLE BROWSER ENVIRONMENT IS AVAILABLE
```

If the available human browser environment has no WebGPU capability, record:

```text
WEB BROWSER RUNTIME = BLOCKED BY ENVIRONMENT / NOT TESTED
```

Do not convert that into PASS or FAIL for Architecture B.

## Comparison probes

At minimum:

```text
Tiny
Panel + Text + Button / minimal transaction path

Spatial
6-card responsive Grid using the MFE-001B headless path
```

A no-UI baseline may be added only when it gives a meaningful same-build comparison and does not distort the experiment.

## Measurement discipline

- use the repository-pinned Rust toolchain;
- run release-mode cost probes for reported timing/allocation data;
- warm up before timed samples;
- report iteration count;
- report median or distribution summary rather than one cherry-picked duration;
- allocation counting must be local to the probe binary and must not alter the GPE library allocator contract;
- do not add a production dependency solely for measurement;
- keep measurement code isolated from production API;
- retain raw observations in the result document.

## Functional regression gate

Before accepting 001C:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
release examples build
Web build/package CI
```

Existing MFE-001A/001B functional tests must remain green.

## Decision

MFE-001C ends with one of:

```text
PASS
PASS WITH CONDITIONS
FAIL
```

Then perform the Architecture B checkpoint:

```text
GO
REVISE
REJECT
```

A `GO` authorizes productionization planning, not immediate API stabilization or consumer migration.

## Explicit non-goals

```text
no styling system implementation
no rounded-rect implementation
no typography fix
no missing-glyph fix
no markup
no SVG
no Taffy
no Arcade migration
no legacy UI removal
no public API freeze
no optimization before measurement
```
