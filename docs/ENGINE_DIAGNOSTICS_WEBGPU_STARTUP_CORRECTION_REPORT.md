# GPE Diagnostics WebGPU Startup Correction Report

Date: **2026-08-28**  
Authority: `ENGINE_DIAGNOSTICS_ARCHITECTURE_CONTRACT.md`, FROZEN v1.0  
Scope: Mission 3.6, WebGPU-unavailable startup non-interference only

## A. Baseline

- expected and observed commit before modification:
  `70b844e027a4cc332a781752b6eb25e878ab797f`;
- branch: `feature/audio2-core`, three commits ahead of its remote tracking
  branch;
- initial worktree: clean; initial `git diff --check`: PASS;
- host: Microsoft Windows 11 Professional 10.0.26100, x64;
- Rust/Cargo: 1.97.1; native target: `x86_64-pc-windows-msvc`; Web target:
  `wasm32-unknown-unknown`;
- browser: retained Mission 3.5 Chromium 140.0.7339.16 / Playwright 1.55.0
  campaign environment;
- pinned Web stack: wgpu 30.0.0, wasm-bindgen 0.2.127, js-sys 0.3.104;
- authority SHA-256 values matched the Mission 3.6 preregistration exactly:
  adversarial review `b764a13a8d83e21cf03b4ac1ea41a40ef7572f9593dbbc4e603985688a222040`,
  architecture contract `dad71c2d3b0beef07ff42d8e5ac0863d30b3193813f9c10ed519c95815659f55`,
  MVP implementation report `804e9480fe27216449aa78281136cb5498ef3474c6fb09e239eec3b0fed5ec94`,
  fault-injection report `f9bb2e66d93be1cd4728306ee8afc3ea7a797f928238b2b6828b21da1dd62778`.

The initial FAIL was reproduced before any production patch with the retained
Mission 3.5 runner and its original OFF/ON artifacts. Chromium again logged
`No available adapters`. OFF failed at `null.requestDevice`; ON failed earlier
at diagnostics-only `null.info`; ON reload repeated `null.info`.

## B. Root cause

Renderer initialization currently performs:

```text
create surface
request adapter
diagnostics ON only: Adapter::get_info()
request device
surface capabilities/configuration
```

The `Adapter::get_info()` result supplies only diagnostic backend, device type,
and bounded opaque name facts. It is not read by any renderer decision.

The pinned wgpu 30 Web backend declares `GPU.requestAdapter()` as a promise of
`JsOption<GpuAdapter>`. In pinned wasm-bindgen 0.2.127, `JsOption` represents
`T | undefined`: only JavaScript `undefined` is absent, while JavaScript `null`
is explicitly a present value. WebGPU returns `null` when no adapter is
available, so that value crosses the binding as `Some(GpuAdapter)` and wgpu
constructs a Rust `Adapter` whose opaque JS reference is null. This explains how
Rust adapter selection can appear successful while the underlying JS adapter is
not usable.

`Adapter::get_info()` has an infallible Rust signature. On the Web backend it
directly reads `self.inner.info()`; the binding-generated JS property access
throws on null, with no `Result` seam at GPE's call site. Native wgpu-core instead
queries a core adapter ID. The demonstrated null-reference mechanism is thus
specific to the Web binding path, although any custom backend could in principle
violate the infallible interface internally.

No adapter fact is already present in GPE without a backend query. Inferring
`BrowserWebGpu`, a device type, or a name merely from the target/request would
fabricate a successfully observed adapter. Moving the read to
`Device::adapter_info()` would defer it until after `request_device`, but on Web
that method is another infallible JS property read and therefore cannot provide
the required fail-open guarantee. The honest Web representation is to leave the
preinitialized adapter fields `Unknown`. Native facts can remain captured at the
existing safe selected-adapter point.

The diagnostics read caused the ON failure before the historical OFF
`request_device` operation, violating `diagnostic loss > engine interference`.
Removing that unsafe Web-only observation does not change renderer selection,
error handling, recovery, or WGPU fatal policy.

## C. Pre-registered correction oracle

The following oracle was recorded before modifying production engine code.

### C01 — Browser WebGPU unavailable parity

PASS only if OFF and ON retain the same engine decision and failure-path class;
ON adds no failure point, does not call `adapter.get_info()` on the unsafe Web
state, fabricates no adapter fact, installs no WGPU policy, and does not fail
earlier because of diagnostics.

### C02 — Native success regression

PASS only if the real primary renderer still works, real adapter facts remain
available where safely observable, lifecycle/present remain coherent, and no
OFF/ON semantic change appears.

### C03 — WASM compile/runtime integrity

PASS only if WASM OFF and ON compile, the retained browser runner reaches the
same historical behavior when WebGPU is unavailable, and no native-only fact is
fabricated.

### C04 — Contract preservation

PASS only if no FROZEN document changes, no `Device::on_uncaptured_error()` is
added, historical `run` remains unchanged, no L3 product is added, and the
`Unknown` / `Unavailable` / `Stale` truth model remains intact.

## D. Minimal correction

Two production concepts changed, both in `src/renderer.rs`:

1. adapter facts are queried only on non-WASM targets;
2. the two native mapping helpers and their imports use the same target guard.

The native `Adapter::get_info()` location and mapping are otherwise unchanged.
On WASM the renderer record's existing initialization leaves backend, device
type, and name as `Unknown`; no replacement value is synthesized. The request
adapter, request device, surface, device-lost callback, renderer lifecycle, and
historical `run` paths are unchanged.

One focused unit test in `src/diagnostics.rs` asserts that skipping adapter
observation leaves all three fields `Unknown` with no value. The pre-existing
`renderer_mutations_update_the_same_retained_record` test continues to assert
that safely supplied adapter facts become known. This is the smallest change
that removes the demonstrated Web-only unsafe query while preserving native
coverage and the existing truth model.

## E. Browser rerun

The retained Mission 3.5 Chromium runner was executed before and after the
patch, using the same campaign-local Chromium 140.0.7339.16 and flags.

| Build / run | Before | After |
|---|---|---|
| OFF | `null.requestDevice` | `null.requestDevice` |
| ON first load | `null.info` | `null.requestDevice` |
| ON reload | `null.info` | `null.requestDevice` |

After correction both OFF and ON logged `No available adapters`, published no
frame result, and failed in wgpu's historical request-device path. The first
error line for both was exactly:

```text
TypeError: Cannot read properties of null (reading 'requestDevice')
```

The ON stack contained zero `reading 'info'` occurrences. C01 is **PASS**:
diagnostics no longer adds an earlier backend access or changes the observable
failure-path class. The engine still fails because the environment has no
usable WebGPU adapter; the correction intentionally does not change that
historical decision.

## F. Native regression check

The real release native probe ran diagnostics ON for 360 deterministic primary
frames and two real tool-window cycles. It exited normally with:

- primary incarnation 1: 360 presents, Vulkan, discrete GPU,
  `NVIDIA GeForce RTX 5060`, destroyed;
- tool incarnations 2 and 3: 50 presents each, the same real adapter facts,
  both destroyed;
- runtime `Ended/NormalExit`;
- native audio `Native/Succeeded`;
- 15 retained events, zero wrap, overwrite, or drop.

The final state was `60f71f7b605df105` and framebuffer hash was
`0e09e073cbbd3ca1`, matching the workload's Mission 3.5 360-frame evidence.
Native adapter capture and renderer semantics are preserved. C02 is **PASS**.

## G. WASM/build validation

| Command / check | Result |
|---|---|
| `cargo check --release --target wasm32-unknown-unknown --example diagnostics_environment_web` | PASS |
| same command with `--features diagnostics` | PASS |
| release OFF and ON WASM builds + `wasm-bindgen` | PASS |
| Chromium negative-environment OFF/ON parity | PASS |
| focused truthful-unknown test | PASS |
| existing safely-observed adapter-facts test | PASS |
| `cargo fmt --all --check` | PASS |
| `cargo test` | PASS — 275 tests |
| `cargo test --all-features` | PASS — 294 tests, 1 existing ignored timing probe |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `git diff --check` | PASS |

WASM does not manufacture `BrowserWebGpu`, device type, or adapter name. The
fields remain truthfully unknown when the guarded backend query is unavailable.
C03 is **PASS**.

## H. Contract impact

**none.** No FROZEN document changed; all four authority hashes remain exactly
the baseline values in section A. No `Device::on_uncaptured_error()`, error
scope, panic hook, parsing, recovery, user-agent detection, Chromium special
case, or L3 product was added. Historical `run` and WGPU fatal policy are
unchanged. The correction directly applies the frozen `unknown > fabricated
certainty` and `diagnostic loss > engine interference` invariants. C04 is
**PASS**. There is no reason to reopen the contract.

## I. Remaining uncertainty

This host still has no WebGPU-capable browser adapter, so successful-browser
adapter/surface observations and successful load/reload remain unverified. The
correction deliberately accepts unknown Web adapter facts rather than claiming
positive browser adapter coverage.

Mission 3.6 does not reclassify the historical items requested out of scope:

```text
E03 E04 E06 E08 E09 E11 E12
F13 F19 F20 F22
```

The positive portion of E10 (120-frame browser renderer success) also could not
be exercised in this no-adapter environment; only the Mission 3.6 negative
startup parity oracle C01 is verified.

## J. Final verdict

**PASS**

The Mission 3.5 FAIL was reproduced and traced to the pinned Web binding's null
semantics. The minimal target-semantic guard removes the diagnostics-only Web
property access. OFF and ON now retain the same historical request-device
failure in the real negative browser environment, while native facts and
renderer behavior remain intact and every requested compile/test/lint gate
passes.

### Mandatory closing questions

1. **Was the Mission 3.5 browser FAIL reproduced before correction?** Yes, with
   Chromium 140: OFF `null.requestDevice`, ON `null.info`.
2. **What is the exact root cause?** WebGPU returns JavaScript `null`; pinned
   wgpu represents the result with wasm-bindgen `JsOption`, whose pinned
   semantics treat only `undefined` as absent. A null opaque adapter therefore
   reaches infallible `Adapter::get_info()` and throws on `null.info`.
3. **Was `Adapter::get_info()` purely diagnostic?** Yes. Its values only fed the
   diagnostic renderer record.
4. **Does the correction change an engine decision?** No.
5. **Does it change fatal WGPU policy?** No.
6. **Do native adapter facts remain available?** Yes; the real Vulkan/NVIDIA
   backend, device type, and name were captured for all three native renderers.
7. **Without browser WebGPU, do OFF and ON now have the same observable failure
   path?** Yes; both fail at `null.requestDevice`, including ON reload.
8. **Is unavailable data now represented honestly?** Yes; Web adapter facts
   remain `Unknown` and contain no fabricated value.
9. **Is the real native workload semantically unchanged?** Yes; lifecycle,
   presents, hashes, adapter facts, audio result, and normal exit are coherent.
10. **Do WASM OFF and ON still compile?** Yes, in release as requested.
11. **Did a FROZEN document change?** No; hashes are unchanged.
12. **Is there a reason to reopen the contract?** No.
13. **Which historical UNVERIFIED items remain unchanged?** E03, E04, E06, E08,
    E09, E11, E12, F13, F19, F20, and F22.

Proposed commit message after human review:

```text
fix: preserve WebGPU startup diagnostics non-interference
```
