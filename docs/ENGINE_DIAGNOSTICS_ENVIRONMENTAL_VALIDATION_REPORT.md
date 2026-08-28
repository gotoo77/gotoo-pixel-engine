# GPE Diagnostics Environmental Validation Report

Date: **2026-08-28**  
Authority: `ENGINE_DIAGNOSTICS_ARCHITECTURE_CONTRACT.md`, FROZEN v1.0  
Scope: Mission 3.5 environmental validation only; no production fix

## A. Baseline

- expected and observed commit before modification: `cf84396108c5554d172f3838c033a7b0ae260ec4`;
- branch: `feature/audio2-core`, two commits ahead of its remote tracking branch;
- initial worktree: clean;
- host: Microsoft Windows 11 Professional 10.0.26100, x64;
- CPU: Intel Core i7-4790K, 4 cores / 8 logical processors;
- RAM: 25,715,789,824 bytes (approximately 24 GiB);
- GPU: NVIDIA GeForce RTX 5060, reported adapter RAM 4,293,918,720 bytes;
- display driver: 32.0.16.1088;
- enumerated audio devices: USB Audio Device, High Definition Audio Device,
  NVIDIA High Definition Audio, and NVIDIA Virtual Audio Device; selected
  CPAL device/backend is to be established by the runtime observation;
- browser: Chromium 140.0.7339.16, installed campaign-locally under `target/`
  through Playwright 1.55.0; tested headless, headed, and headed with SwiftShader
  explicitly allowed;
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.6;
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`;
- host target: `x86_64-pc-windows-msvc`;
- native A/B profile: release for both sides; diagnostic inspection smoke may
  additionally use debug;
- WASM target: `wasm32-unknown-unknown`, release for both OFF and ON;
- A: default features, historical `run`; B: `diagnostics`, explicit enabled
  handle and `run_with_diagnostics`;
- initial SHA-256, adversarial review:
  `b764a13a8d83e21cf03b4ac1ea41a40ef7572f9593dbbc4e603985688a222040`;
- initial SHA-256, frozen contract:
  `dad71c2d3b0beef07ff42d8e5ac0863d30b3193813f9c10ed519c95815659f55`.

## B. Environmental harness

The preregistered harness consists of two consumer examples:

- `diagnostics_environment_probe`: an auto-terminating native GPE window using
  a deterministic tick-driven framebuffer, a real primary renderer, optional
  real tool renderer create/destroy/recreate cycles, and normal platform-audio
  initialization. The same source is compiled OFF and ON. It emits the final
  deterministic state/framebuffer hashes; the ON build also emits the owned L1
  observation after clean runtime teardown.
- `diagnostics_environment_web`: the same deterministic framebuffer principle
  in a browser-targeted example. It publishes a bounded consumer-owned result
  to the browser global object and console after real frame progression. OFF
  uses historical `run`; ON uses the explicit enabled registration and includes
  a real L1 observation. Reload creates a fresh page/runtime.

No probe calls a private producer, simulates a backend event, installs a panic
or uncaptured-error handler, parses WGPU panic text, or changes engine behavior.
OS process timing and working-set observations are external L3-style evidence
only and do not create a product L3.

## C. Pre-registered E01-E12 oracles

These oracles were written before the first native or browser execution.

### E01 — native primary renderer real path

PASS only if a real native window reaches `Running`, retains one primary record
with a real bounded adapter name, backend and device type, records a non-zero
surface configuration and advancing presents, exits normally, and ends with the
renderer `Destroyed`. Any contradiction with visible/runtime facts is FAIL.
Missing interactive/backend execution is UNVERIFIED.

### E02 — real tool renderer lifecycle

PASS only if real tool windows are created, destroyed, recreated and destroyed,
with distinct primary/tool roles, strictly increasing non-reused incarnations,
destroyed historical records, and presents on each created renderer. A role or
incarnation reuse/misattribution is FAIL. If real tool creation is unavailable,
the result is UNVERIFIED.

### E03 — real renderer teardown / late delivery evidence

PASS applies only to the actually exercised teardown facts: clean real renderer
destruction, stale historical surface data and no observed misattribution. A
real late callback attributed to a new incarnation is FAIL. If no real late
callback occurs, the late-delivery property and F20 remain UNVERIFIED even when
ordinary teardown passes.

### E04 — real WGPU device-lost callback

PASS only if WGPU itself safely delivers the already-installed callback during
an otherwise contract-compatible run, with the originating incarnation. No
callback means UNVERIFIED. No driver reset, GPU exhaustion or new handler is
permitted.

### E05 — real WGPU validation / surface behavior

PASS only for categories actually returned by a real surface/backend. Normal
surface configuration and present progression may PASS independently. Absence
of a surface error does not verify any unexercised category. A real category
that diagnostics maps incorrectly is FAIL.

### E06 — F13 / real WGPU OOM

UNVERIFIED unless an existing safe, local, contract-compatible mechanism causes
WGPU to deliver an exact OOM category. GPU/host exhaustion and
`Device::on_uncaptured_error()` are forbidden. If observation would require the
handler/policy to change, record `HUMAN DECISION REQUIRED`; do not execute it.

### E07 — native audio initialization

PASS only if a native run records the actually selected `Native/Succeeded` or
the actual `Noop/FailedFallback` outcome, contains no gameplay/BGM/playback
detail, and the renderer workload completes without diagnostic-induced audio
failure. A fabricated native success or semantic audio data is FAIL.

### E08 — real CPAL callback/error path

PASS only for a real CPAL callback or real backend failure actually observed and
correctly categorized. A healthy environment that emits no error is recorded
as `ENVIRONMENT DID NOT PRODUCE ERROR`; the callback property remains
UNVERIFIED. No device disruption is permitted.

### E09 — audio environment variability

PASS only if two genuinely distinct real selections/outcomes are executed (for
example native success and real unavailable fallback). A synthetic producer or
the same device twice is insufficient. Otherwise UNVERIFIED.

### E10 — browser runtime smoke

PASS only if both OFF and ON run in a real browser, start asynchronously, create
a real renderer/surface, advance at least 120 deterministic frames, remain
responsive, publish the completion marker, and introduce no console panic.
Compile-only evidence is UNVERIFIED.

### E11 — browser observation semantics

PASS only if the ON browser returns a real L1 observation with `Running`, a
primary renderer, real available adapter/backend facts where the browser exposes
them, bounded text, real surface/present progression, Web audio semantics, and
no fabricated native-only tool/CPAL data. Any unavailable fact presented as
known is FAIL.

### E12 — browser teardown / reload

PASS only if at least one load/run/reload cycle creates fresh runtime state and
both loads remain free of diagnostic double-registration or panic. This does not
claim proof of browser-memory leak absence. If browser execution/reload is not
available, UNVERIFIED.

## D. Results matrix

| ID | Environment | Executed | Result | Evidence | Remaining uncertainty |
|---|---|---:|---|---|---|
| E01 | Windows native, Vulkan/NVIDIA | YES | PASS | real primary incarnation 1; Vulkan/discrete RTX 5060; BGRA8 sRGB/FIFO/opaque 960x540; 360 presents; `Destroyed`; normal exit | physical screen visibility is not claimed |
| E02 | Windows native tool windows | YES | PASS | real tool incarnations 2 then 3; 50 presents each; both `Destroyed`; primary remained distinct | no backend callback occurred during teardown |
| E03 | Windows real renderer teardown | PARTIAL | UNVERIFIED | primary and both tools destroyed cleanly; stale-on-destroy facts correct; no misattribution observed | no real late callback was delivered, so the target late-delivery property was not exercised |
| E04 | Windows WGPU/Vulkan | YES | UNVERIFIED | device-lost stayed `Unknown` through smoke and 10-minute run | real device-lost callback not produced |
| E05 | Windows WGPU/Vulkan surface | YES | PASS | real surface configured and 360/50/50 presents progressed; 36,000-present long-run; no surface fault occurred | individual failure categories remain unexercised |
| E06 | Windows/NVIDIA | NO | UNVERIFIED | no safe contract-compatible OOM method exists | F13 frozen Option A gap; testing would require a human policy decision |
| E07 | Windows native audio | YES | PASS | actual observation `Native/Succeeded`; generic error remained `Unknown`; no gameplay/BGM/playback detail; normal workload exit | selected device name is intentionally outside the MVP |
| E08 | Windows CPAL | YES | UNVERIFIED | `ENVIRONMENT DID NOT PRODUCE ERROR` in smoke or 10-minute run | real callback/error delivery remains unexercised |
| E09 | Windows audio inventory | PARTIAL | UNVERIFIED | one real native-success environment executed | no second genuinely distinct real selection/fallback was safely available |
| E10 | Chromium 140, Windows, no WebGPU adapter | YES | FAIL | OFF throws at `null.requestDevice`; ON throws earlier at `null.info` from diagnostic `adapter.get_info()`; no frame marker; three browser modes attempted | a WebGPU-capable browser environment was not available for the positive smoke |
| E11 | Chromium 140, Windows | YES | UNVERIFIED | ON startup attempted in a real browser but no L1 observation became reachable | adapter/surface/browser availability semantics after successful init remain unknown |
| E12 | Chromium 140 reload | YES | UNVERIFIED | ON reload reproduced the same startup exception without a double-registration signature | no successful load/run/reload cycle occurred; no browser-memory claim |

## E. F13 / F19 / F20 / F22 impact

| Fault | Previous status | New evidence | New status | Reason |
|---|---|---|---|---|
| F13 | UNVERIFIED | no safe existing OOM observation seam; no exhaustion attempted | UNVERIFIED | frozen Option A remains decisive; changing it requires human review |
| F19 | UNVERIFIED | real Chromium `No available adapters` startup failure; OFF/ON fail at different JS calls | UNVERIFIED | one real browser stage is verified, but the full native/WGPU/audio startup matrix is not; the observed difference is a FAIL finding, not sufficient breadth to reclassify F19 |
| F20 | UNVERIFIED | real primary/tool destruction and recreate were exercised; no late callback arrived | UNVERIFIED | ordinary teardown passed, but real late-callback attribution was not exercised |
| F22 | UNVERIFIED | 5 release repetitions per side after warm-up; all tick/state/framebuffer hashes equal; timing distributions overlap but ON median is 1.81% higher | UNVERIFIED | semantic equivalence is strongly verified for this native workload, but end-to-end timing does not directly validate the frozen per-capture p99 budget and no OFF long-run memory control was run |

## F. Native WGPU findings

The authoritative native ON smoke ran 360 primary frames and two real tool
cycles. Diagnostics reported:

- primary `(Primary, 1)`, Vulkan, discrete GPU, bounded 23-byte adapter name
  `NVIDIA GeForce RTX 5060`, not truncated;
- surface BGRA8 sRGB, FIFO, opaque, 960x540, last present 360;
- tool `(Tool, 2)` then `(Tool, 3)`, each Vulkan/discrete, 480x300 and last
  present 50;
- all three lifecycle records `Destroyed`, with last surface/present facts stale
  for `RendererDestroyed` in the full observation;
- no surface failure, device lost or controlled WGPU error was observed;
- event history retained 15 records with zero overwrite/drop.

The 10-minute ON run used only the primary renderer and reached 36,000 presents,
normal `Ended/NormalExit`, zero diagnostic degradation, no surface/WGPU error,
one renderer identity and no record/event loss.

No real device-lost callback, late callback, validation failure or OOM occurred.
Their absence is not converted into a negative fact.

## G. Native audio / CPAL findings

Both the smoke and 10-minute run recorded `AudioBackend::Native` and
`AudioInitializationOutcome::Succeeded`. The last generic backend error remained
`Unknown`, which means `ENVIRONMENT DID NOT PRODUCE ERROR`, not “CPAL errors are
absent”. No BGM, asset, playback, device-name or gameplay data was captured.

Only one selected environment was available. Windows enumerated four audio
endpoints, but the frozen public summary intentionally does not expose which
default device CPAL selected. No safe second real unavailable/fallback
environment was manufactured; E08 and E09 remain unverified.

## H. WASM/browser findings

Both OFF and ON WASM examples compiled in release and were processed by
`wasm-bindgen 0.2.127`. Chromium 140.0.7339.16 was then run headless, headed, and
headed with `--use-angle=swiftshader`, `--enable-unsafe-swiftshader`,
`--enable-unsafe-webgpu`, and `--ignore-gpu-blocklist` as applicable. Every mode
logged `No available adapters`.

The corrected authoritative runner ignores winit's documented control-flow
exception and waits for the real asynchronous outcome. In that run:

- OFF raised `TypeError: Cannot read properties of null (reading
  'requestDevice')` in WGPU's Web adapter request-device path;
- ON raised `TypeError: Cannot read properties of null (reading 'info')` at
  `Renderer::new_inner`'s diagnostics-only `adapter.get_info()` call;
- ON reload raised the same `null.info` exception again;
- no deterministic 120-frame result or L1 browser observation was published.

This is a real environment/startup finding. Diagnostics changes the observable
failure site and exception in the WebGPU-unavailable browser. It is classified
FAIL for E10/non-interference. No production correction is made. Since no
WebGPU-capable browser runtime was reached, positive E10-E12 semantics and
browser timing/memory remain unverified.

## I. A/B diagnostics OFF/ON

Predeclared temporal methodology: one warm-up process per side followed by at
least five release repetitions per side, identical tick count/tool schedule and
machine. Semantic gate: identical final tick, state hash and framebuffer hash.
Performance evidence will report median and p95 when the sample supports it and
will not reinterpret system noise. The frozen per-frame budget remains 0.5 us
p99 native / 1 us p99 WASM; end-to-end process duration is supporting evidence,
not a direct substitute for that microbenchmark gate.

The release workload used 240 deterministic ticks and the same two tool cycles.
Warm-up hashes matched. All five measured OFF and all five measured ON runs
reported:

```text
ticks=240
state_hash=7e6c1ceebd281009
framebuffer_hash=3429409088ed70a1
```

Durations in microseconds:

| Side | Samples | Median | p95 (nearest-rank) |
|---|---|---:|---:|
| OFF | 5,367,136; 5,373,203; 5,386,103; 5,435,743; 5,467,324 | 5,386,103 | 5,467,324 |
| ON | 5,415,465; 5,463,016; 5,483,406; 5,484,021; 5,930,452 | 5,483,406 | 5,930,452 |

The ON median is 97,303 us / 1.81% higher. The distributions overlap and the ON
p95 contains one system/backend outlier. This end-to-end measure includes three
adapter/device/surface initializations, window scheduling, vsync, audio init and
teardown; dividing it by frames would not be a credible measurement of the
diagnostic capture budget. Therefore semantic equivalence is PASS for this
native workload, while performance equivalence against the frozen per-capture
p99 budget remains UNVERIFIED. The browser A/B did not reach a renderer and
instead produced the E10 failure above.

## J. Long-run / memory findings

Predeclared OS evidence: sample the real process working set after warm-up during
an enabled workload lasting 10 minutes when feasible. A valid conclusion is
limited to whether monotonic growth is observed over that interval; fixed Rust
layout, allocator allocation count, process working set and GPU/browser memory
remain distinct.

The ON primary-only release workload ran 36,000 presents for 602.443 seconds.
Twenty OS samples were collected at 30-second intervals:

- working set: initial 159,997,952 B; minimum 136,728,576 B; maximum
  162,742,272 B; final 136,888,320 B;
- private bytes: initial 364,572,672 B; minimum 364,400,640 B; maximum
  371,843,072 B; final 371,638,272 B;
- CPU time at the final sample: 68,156 ms;
- diagnostic store outcome: one retained renderer identity, nine retained rare
  events, zero wrap/overwrite/drop, no degradation, normal exit.

Private bytes moved once to a roughly 371 MiB plateau and later fluctuated down;
working set was eventually trimmed by about 26 MiB. **No monotonic growth was
observed over 10 minutes.** This is process-level evidence only. Without an OFF
long-run control it cannot attribute the one plateau change to diagnostics, and
it says nothing about GPU or browser memory.

## K. Unexpected findings

1. The first native exploratory run accidentally varied the consumer tool-window
   title with every tick. Because title is part of the requested config, this
   created 101 real renderer incarnations. Identities remained monotone and
   bounded retention/loss metadata behaved correctly, but this was not the
   preregistered two-cycle workload. The harness title was made constant and the
   authoritative run then produced exactly incarnations 1, 2 and 3. No engine
   code was changed.
2. Chromium's WGPU Web backend logged `No available adapters` yet exposed a
   wrapper that reached a null JS adapter. The historical path failed later at
   `requestDevice`; the diagnostics path failed earlier at `get_info`. This is
   the campaign's substantive FAIL finding.
3. Working set and private bytes diverged during the long-run: Windows trimmed
   working set while private bytes stayed on a plateau. Reporting only one would
   have produced a misleading memory narrative.

## L. Contract impact

No frozen decision is falsified and no contract revision is required. The
browser finding is compatible with the contract's requirement that adapter facts
be captured after selection, but it falsifies environmental non-interference of
the current implementation when WGPU's Web adapter wrapper contains a null JS
adapter. The appropriate next action is implementation diagnosis/correction in a
separately authorized mission, not a contract change. Fatal WGPU policy remains
unchanged; no `Device::on_uncaptured_error()` exists.

## M. Final verdict

**FAIL**

Native WGPU/tool/audio, deterministic native semantics and the 10-minute
long-run produced strong positive evidence. Nevertheless, a real Chromium
startup failure is observably different with diagnostics ON: the
diagnostics-only adapter-info read throws before the historical request-device
failure. Mission 3.5 forbids correcting and retrying this into a PASS. The FAIL
does not require contract revision.

### Mandatory closing questions

1. **Did diagnostics modify observable behavior of a real renderer?** No on the
   successfully running native renderer; yes on the real browser renderer
   startup path before a renderer became ready.
2. **Did a real WGPU observation contradict synthetic producers?** No native
   category contradicted them. The browser null-adapter behavior exposed an
   environmental path absent from the synthetic startup matrix.
3. **Was a real device-lost callback observed?** No.
4. **Was a real CPAL/audio error observed?** No; the environment did not produce
   one.
5. **Did real teardown produce a wrong renderer incarnation?** No.
6. **Was a real late callback observed?** No.
7. **Did browser WASM work with diagnostics enabled?** No; no WebGPU adapter was
   available and ON threw at `adapter.get_info()`.
8. **Was unavailable data presented as known in a real environment?** No
   materialized observation did so; browser startup failed before observation.
9. **Did native A/B show semantic divergence?** No; all hashes matched.
10. **Did A/B show a measurable regression exceeding the budgets?** A 1.81%
    median end-to-end difference was measured, but it is not a credible direct
    measure of the frozen per-capture p99 budget; budget exceedance remains
    unverified.
11. **Was diagnostics-attributable monotonic memory growth observed?** No.
12. **Can F13 be tested cleanly without reopening frozen WGPU policy?** No.
13. **Can F19 be reclassified?** No; one real browser stage is insufficient and
    also revealed a FAIL behavior difference.
14. **Can F20 be reclassified?** No; no real late callback occurred.
15. **Can F22 be reclassified?** No; semantic equivalence is strongly verified
    for the native workload, but frozen-budget performance and OFF/ON long-run
    memory equivalence are not fully established.
16. **Does evidence require reopening the FROZEN contract?** No.

## N. Recommended next step

Human review of this FAIL report, followed by a separately authorized, narrowly
scoped diagnosis/correction mission for WebGPU-unavailable startup parity and a
rerun on a known WebGPU-capable browser host. Preserve F13/F19/F20/F22 as
UNVERIFIED until their missing evidence is supplied. Do not begin Mission 4,
Void Canticle integration or product L3 architecture.

Proposed commit message after human review:

```text
test: add diagnostics environmental validation campaign
```

At report finalization, no commit had yet been created; subsequent human review
authorized the focused Mission 3.5 commit proposed above.

## O. Final validation and preservation record

Recovery inspection confirmed that `HEAD` remains exactly
`cf84396108c5554d172f3838c033a7b0ae260ec4`. The completed long-run process is
no longer alive; unrelated `cargo run` processes observed during recovery
belong to `I:\dev\gpe_druid_reborn` and were left untouched. The persisted
long-run stdout records 36,000 presents, `Ended/NormalExit`, and an empty stderr.
The OS samples were consolidated into section J before the editor crash; they
were not separately redirected to an artifact. Because the final observation
and consolidated result are coherent and exploitable, the long-run was not
repeated.

Final validations on the reported worktree:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo test` | PASS — 275 tests |
| `cargo test --all-features` | PASS — 293 tests, 1 ignored manual timing probe |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo check --release --target wasm32-unknown-unknown --example diagnostics_environment_web` | PASS |
| same command with `--features diagnostics` | PASS |
| `git diff --check` | PASS |

The first recovery-time `cargo test` exposed that the new browser consumer used
the WASM-only `js_sys` dependency while Cargo also compiled the example for the
native test target. The harness was scoped with a target guard around that
browser-global publication; no engine code, observation semantics, oracle or
result changed. The subsequent complete validation passed.

Authority-document SHA-256 values at closure:

- adversarial review (FROZEN):
  `b764a13a8d83e21cf03b4ac1ea41a40ef7572f9593dbbc4e603985688a222040`;
- architecture contract (FROZEN):
  `dad71c2d3b0beef07ff42d8e5ac0863d30b3193813f9c10ed519c95815659f55`;
- MVP implementation report:
  `804e9480fe27216449aa78281136cb5498ef3474c6fb09e239eec3b0fed5ec94`;
- fault-injection report:
  `f9bb2e66d93be1cd4728306ee8afc3ea7a797f928238b2b6828b21da1dd62778`.

Mission 3.5 changes are limited to `Cargo.toml`, the two consumer examples and
this report. No production engine source or FROZEN document was modified. At
report finalization, the focused commit was awaiting human authorization.

# STOP
