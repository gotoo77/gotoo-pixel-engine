# GPE Diagnostics Fault-Injection Report

Date: **2026-08-28**  
Authority: `ENGINE_DIAGNOSTICS_ADVERSARIAL_REVIEW.md` F01-F22 and FROZEN contract v1.0  
Scope: Mission 3 adversarial validation only; no production fix

## A. Baseline

- expected and observed commit before modification: `10b1d8f2ea97c48db1d5966aa61951de825201f0`;
- branch: `feature/audio2-core`, one commit ahead of its remote tracking branch;
- initial worktree: clean;
- host: Windows 11 Professional 10.0.26100, x64, Intel64 Family 6 Model 60,
  approximately 24 GiB RAM;
- toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.6,
  `cargo 1.97.1 (c980f4866 2026-06-30)`, host
  `x86_64-pc-windows-msvc`;
- primary campaign artifact: native debug with
  `diagnostics-fault-injection` (which implies `diagnostics`);
- F06 artifact: separate native debug build with `RUSTFLAGS=-C panic=abort`;
- F22 build controls: default feature-absent build and a diagnostics-compiled
  runtime-disabled handle;
- initial SHA-256, adversarial review: `b764a13a8d83e21cf03b4ac1ea41a40ef7572f9593dbbc4e603985688a222040`;
- initial SHA-256, frozen contract: `dad71c2d3b0beef07ff42d8e5ac0863d30b3193813f9c10ed519c95815659f55`.

The SHA-256 values above are the pre-campaign preservation reference. They are
not the abbreviated Git object hashes quoted by the Mission 2 report.

## B. Harness

The harness is a dedicated feature-gated `diagnostics_fault_probe` binary. Its
controller starts exactly
one sacrificial child per scenario, applies a scenario-specific bounded timeout,
captures stdout/stderr and exit status, kills only that child when required, and
classifies the consumer artifact as missing, partial, or successful by an
explicit `END REPORT Fxx` footer. The controller is an experimental supervisor;
it is not product LEVEL 3.

The seam is compiled only by `diagnostics-fault-injection`, a default-off feature
which implies `diagnostics`; it is also excluded from WASM. It is a child-probe
module nested inside `diagnostics.rs`, so it can exercise the exact private
producer/store paths without making engine writers public. Normal builds have no
fault branch, failpoint allocator, controller, panic hook, filesystem writer or
new callback. No `Device::on_uncaptured_error()` call was added.

Controller classifications on this Windows host are limited to facts the OS and
Rust expose: normal exit, conventional panic-unwind exit 101, other abnormal
status, controller-initiated termination, timeout, and artifact completeness.
The controller uses scenario knowledge plus stderr to distinguish the re-panic
abort from explicit `process::abort`; the Windows exit status alone is identical.
It does not claim portable signal names.

Timeouts were 5 seconds normally, 1 second for F09/F10, 10 seconds for F18 and
15 seconds for F17. F08 was killed only after its readiness artifact existed.
Every child wrote into a unique directory. F17 used 1,000,000 iterations. F18
used four producers, 100 rounds and 250 publications per producer/round
(100,000 total), a bounded repetition count chosen to create contention while
remaining well below the timeout.

## C. Pre-registered oracles F01-F22

All oracles below were written before executing any fault. Common forbidden
effects are: unbounded host resource use, backend query during degraded read,
consumer callback from CAPTURE, mutation of engine decisions, hidden retry, and
changing WGPU's fatal policy. Unless stated otherwise, timeout is 2 seconds and
`UNVERIFIED` is required when only a synthetic producer path is available but a
real backend/runtime fact is claimed.

### F01 — panic on the running main path

- **Fault injected:** Rust unwind panic after bootstrap and `Running` capture.
- **Expected observable facts:** consumer panic payload/location, build facts,
  running lifecycle and a primary renderer incarnation from the shared store.
- **Expected unavailable facts:** GPU root cause and inter-thread causality.
- **Expected process outcome:** Rust panic-unwind termination.
- **Expected diagnostic outcome:** a best-effort hook report may be successful.
- **Allowed information loss:** contended sections and final flush.
- **Forbidden effects:** wait, diagnostic second panic, changed panic decision.
- **PASS / FAIL / UNVERIFIED:** PASS iff termination and cached facts match and
  the child does not hang; FAIL on forbidden effects or fabricated facts;
  UNVERIFIED if the host cannot distinguish the panic exit.

### F02 — worker panic after bootstrap

- **Fault injected:** a named worker panics; the main thread joins it.
- **Expected observable facts:** worker identity/payload and accessible partial
  cache; the engine is not forced to terminate by GPE diagnostics.
- **Expected unavailable facts:** a renderer frame association for the worker.
- **Expected process outcome:** normal exit after the consumer handles `JoinError`.
- **Expected diagnostic outcome:** hook observation from the same bounded store.
- **Allowed information loss:** final consumer report is best-effort.
- **Forbidden effects:** GPE-imposed engine/process exit, wait or false frame.
- **PASS / FAIL / UNVERIFIED:** PASS iff worker panic is observed and child can
  continue normally; FAIL on interference; UNVERIFIED if thread panic hooks are
  unavailable on the target.

### F03 — panic/read while one diagnostic section is contended

- **Fault injected:** renderer section held while an L2 read is attempted.
- **Expected observable facts:** renderer `UNAVAILABLE`, other sections attempted,
  degraded status and bounded read latency.
- **Expected unavailable facts:** the locked renderer section.
- **Expected process outcome:** normal probe exit.
- **Expected diagnostic outcome:** partial L2 observation from the L1 store.
- **Allowed information loss:** all facts in the contended section.
- **Forbidden effects:** wait/retry, poison cascade, backend query.
- **Timeout:** L2 read itself must complete below 100 ms; child 2 seconds.
- **PASS / FAIL / UNVERIFIED:** PASS iff local unavailability is returned within
  the read bound; FAIL on wait or cross-section loss; UNVERIFIED if timing cannot
  be measured monotonically.

### F04 — formatter failure

- **Fault injected:** consumer formatter returns an error after CAPTURE.
- **Expected observable facts:** capture before and after the formatter error is
  unchanged except for explicitly injected engine events.
- **Expected unavailable facts:** formatted output/footer from the failed attempt.
- **Expected process outcome:** normal controlled exit.
- **Expected diagnostic outcome:** formatter failure remains outside GPE.
- **Allowed information loss:** the failed PRESENT attempt.
- **Forbidden effects:** recursion, new diagnostic event, engine/store mutation.
- **PASS / FAIL / UNVERIFIED:** PASS iff capture remains readable and event count
  does not change; FAIL on feedback; UNVERIFIED if separation cannot be observed.

### F05 — hook/formatter re-panic

- **Fault injected:** consumer panic hook records one guarded attempt then panics.
- **Expected observable facts:** at most the first guarded breadcrumb.
- **Expected unavailable facts:** reliable final report.
- **Expected process outcome:** abnormal fatal termination, not a hang.
- **Expected diagnostic outcome:** no claim that GPE recovers or finalizes.
- **Allowed information loss:** all final output.
- **Forbidden effects:** recursive loop, deadlock or GPE policy change.
- **PASS / FAIL / UNVERIFIED:** PASS iff the child terminates within the bound and
  no repeated diagnostic loop occurs; FAIL on hang/repetition; UNVERIFIED if the
  runtime suppresses hook execution without observable evidence.

### F06 — panic=abort

- **Fault injected:** Rust panic in an artifact compiled with `panic=abort`.
- **Expected observable facts:** minimal hook attempt and cached facts may exist.
- **Expected unavailable facts:** unwind, destructors and reliable flush.
- **Expected process outcome:** abnormal abort termination.
- **Expected diagnostic outcome:** best-effort pre-abort artifact only.
- **Allowed information loss:** partial or missing final report.
- **Forbidden effects:** claim of unwind/completeness or timeout.
- **PASS / FAIL / UNVERIFIED:** PASS only when the artifact proves abort strategy,
  terminates abnormally and makes no unwind claim; FAIL otherwise; UNVERIFIED if
  a separate abort artifact cannot be built safely.

### F07 — `process::abort`

- **Fault injected:** explicit abort after a consumer breadcrumb.
- **Expected observable facts:** prewritten breadcrumb and external exit status.
- **Expected unavailable facts:** panic hook, destructors, footer/final report.
- **Expected process outcome:** abnormal explicit-abort termination.
- **Expected diagnostic outcome:** `NO FINAL REPORT` is correct.
- **Allowed information loss:** everything not persisted before abort.
- **Forbidden effects:** waiting for or fabricating a final report.
- **PASS / FAIL / UNVERIFIED:** PASS iff abnormal exit occurs promptly with only
  prior breadcrumb; FAIL on footer/hang; UNVERIFIED if abort is intercepted.

### F08 — controlled external termination

- **Fault injected:** controller terminates its exact child PID after readiness.
- **Expected observable facts:** pre-termination breadcrumb and external action.
- **Expected unavailable facts:** hook and final report.
- **Expected process outcome:** externally terminated child.
- **Expected diagnostic outcome:** `NO FINAL REPORT` is correct.
- **Allowed information loss:** all post-breadcrumb data.
- **Forbidden effects:** killing another process or claiming GPE interception.
- **PASS / FAIL / UNVERIFIED:** PASS iff only the child is terminated and artifact
  stays partial; FAIL on collateral effect/final fabrication; UNVERIFIED if exact
  child termination cannot be controlled.

### F09 — hang

- **Fault injected:** bounded child-only non-progress loop after heartbeat.
- **Expected observable facts:** last heartbeat and controller timeout.
- **Expected unavailable facts:** exact in-process cause and final report.
- **Expected process outcome:** timeout followed by exact-child termination.
- **Expected diagnostic outcome:** prior breadcrumb only.
- **Allowed information loss:** terminal state.
- **Forbidden effects:** in-process watchdog claim or blocked controller.
- **Timeout:** 1 second plus bounded kill/wait margin.
- **PASS / FAIL / UNVERIFIED:** PASS iff timeout is detected and cleanup completes;
  FAIL if controller hangs; UNVERIFIED if exact-child kill is unavailable.

### F10 — deliberate deadlock

- **Fault injected:** child blocks forever on a deliberately held private mutex.
- **Expected observable facts:** last heartbeat and controller timeout.
- **Expected unavailable facts:** stacks and final report.
- **Expected process outcome:** external timeout/termination.
- **Expected diagnostic outcome:** no L2 guarantee is claimed for total deadlock.
- **Allowed information loss:** terminal observation.
- **Forbidden effects:** deadlocking the controller/test runner.
- **Timeout:** 1 second plus bounded kill/wait margin.
- **PASS / FAIL / UNVERIFIED:** PASS iff isolated timeout cleanup succeeds; FAIL if
  parent blocks; UNVERIFIED if isolation is unavailable.

### F11 — controlled allocation failure at materialization

- **Fault injected:** allocator rejects allocations only around materialization.
- **Expected observable facts:** either zero-allocation read succeeds from fixed
  state or child terminates with an honest missing report.
- **Expected unavailable facts:** owned text/report if allocation were required.
- **Expected process outcome:** normal for current zero-allocation materializer.
- **Expected diagnostic outcome:** capture is unchanged; no backend query.
- **Allowed information loss:** final materialization under actual allocation need.
- **Forbidden effects:** host RAM exhaustion or unbounded allocation.
- **PASS / FAIL / UNVERIFIED:** PASS iff the controlled failpoint stays host-safe
  and current materialization succeeds without allocation; FAIL on engine/store
  mutation; UNVERIFIED for real system OOM behavior.

### F12 — WGPU validation synthetic producer path

- **Fault injected:** synthetic surface-validation outcome at the authorized
  diagnostics producer, not a real uncaptured backend callback.
- **Expected observable facts:** Validation plus renderer provenance.
- **Expected unavailable facts:** actual callback thread, backend cause/delivery.
- **Expected process outcome:** normal synthetic probe exit.
- **Expected diagnostic outcome:** `SYNTHETIC PATH VERIFIED` only.
- **Allowed information loss:** none in uncontended synthetic path.
- **Forbidden effects:** formatter/backend call or uncaptured handler install.
- **PASS / FAIL / UNVERIFIED:** PASS iff synthetic mapping is exact; FAIL on wrong
  provenance/category; real backend callback remains explicitly UNVERIFIED.

### F13 — WGPU OOM simulated event

- **Fault injected:** request to inject a synthetic WGPU OOM category.
- **Expected observable facts:** canonical review asks for OOM plus incarnation.
- **Expected unavailable facts:** real GPU-memory exhaustion.
- **Expected process outcome:** normal if a supported seam exists.
- **Expected diagnostic outcome:** must not install an uncaptured handler or invent
  a category absent from the frozen MVP.
- **Allowed information loss:** uncaptured OOM is unknown under frozen Option A.
- **Forbidden effects:** GPU exhaustion, text parsing or WGPU policy change.
- **PASS / FAIL / UNVERIFIED:** UNVERIFIED if no contract-compatible producer seam
  exists; FAIL if certainty is fabricated or policy changed; PASS only for an
  already-authorized exact synthetic producer.

### F14 — simulated device-lost path

- **Fault injected:** synthetic invocation of the existing device-lost producer.
- **Expected observable facts:** bounded reason/message and original incarnation.
- **Expected unavailable facts:** physical device recovery and real delivery.
- **Expected process outcome:** normal synthetic probe exit.
- **Expected diagnostic outcome:** `SYNTHETIC PATH VERIFIED` only.
- **Allowed information loss:** callback drop under contention.
- **Forbidden effects:** WGPU API call from callback path or current-role rewrite.
- **PASS / FAIL / UNVERIFIED:** PASS iff synthetic facts/provenance/truncation are
  correct; FAIL on backend work or misattribution; real callback is UNVERIFIED.

### F15 — Lost then Outdated surface failures

- **Fault injected:** exact synthetic acquisition variants Lost then Outdated.
- **Expected observable facts:** two distinct historical categories and exact
  renderer provenance; last failure is Outdated.
- **Expected unavailable facts:** physical screen visibility.
- **Expected process outcome:** normal probe exit.
- **Expected diagnostic outcome:** no collapse to `SurfaceChanged` in diagnostics.
- **Allowed information loss:** overwrite only if capacity is intentionally hit.
- **Forbidden effects:** category collapse/fabrication.
- **PASS / FAIL / UNVERIFIED:** PASS iff both distinct records are retained in
  order and scalar is exact; FAIL otherwise; real surface faults are UNVERIFIED.

### F16 — synthetic audio backend error

- **Fault injected:** generic stream error at the diagnostics producer.
- **Expected observable facts:** audio category/backend/stamp without frame.
- **Expected unavailable facts:** physical device state and real CPAL context.
- **Expected process outcome:** normal synthetic probe exit.
- **Expected diagnostic outcome:** synthetic producer path only.
- **Allowed information loss:** drop under contention.
- **Forbidden effects:** required `eprintln`, backend call or playback detail.
- **PASS / FAIL / UNVERIFIED:** PASS iff generic facts are exact and bounded; FAIL
  on forbidden work/data; real CPAL/backend failure remains UNVERIFIED.

### F17 — ring/store saturation and long run

- **Fault injected:** 1,000,000 deterministic rare events, plus counters seeded at
  their overflow boundary where appropriate.
- **Expected observable facts:** ring retained=64, exact overwrite count, fixed
  physical store/observation size, zero steady diagnostic allocations, saturation
  at `u64::MAX` with flag.
- **Expected unavailable facts:** complete event history.
- **Expected process outcome:** normal bounded exit.
- **Expected diagnostic outcome:** oldest overwrite with loss metadata.
- **Allowed information loss:** all but the newest 64 records.
- **Forbidden effects:** Vec/String growth, wrap, panic or consumer dependence.
- **Timeout:** 15 seconds.
- **PASS / FAIL / UNVERIFIED:** PASS iff exact counts, allocation and capacities
  hold; FAIL on growth/wrap/corruption; RSS-level allocator fragmentation remains
  UNVERIFIED if no credible per-process measurement is available.

### F18 — concurrent producers

- **Fault injected:** four producers, 100 bounded rounds of 250 publications each.
- **Expected observable facts:** complete typed records, bounded retained history,
  declared insertion order and best-effort loss counts.
- **Expected unavailable facts:** total causality and physical occurrence order.
- **Expected process outcome:** normal exit in 10 seconds.
- **Expected diagnostic outcome:** drop rather than global wait.
- **Allowed information loss:** contention drops and ring overwrites.
- **Forbidden effects:** torn record, deadlock, unbounded queue, causal claim.
- **PASS / FAIL / UNVERIFIED:** PASS across the documented repetitions iff joins
  finish and invariants hold; FAIL on hang/torn data; scheduler coverage remains
  inherently non-exhaustive, not a reason to fabricate certainty.

### F19 — failures through startup stages

- **Fault injected:** controlled diagnostics-producer failures for pre-renderer,
  create-surface, request-adapter and request-device stages.
- **Expected observable facts:** build, honest lifecycle and only already-reached
  renderer facts/categories.
- **Expected unavailable facts:** future subsystem facts.
- **Expected process outcome:** controlled normal probe exit.
- **Expected diagnostic outcome:** synthetic stage matrix, no default values.
- **Allowed information loss:** platform/backend facts not actually reached.
- **Forbidden effects:** invented adapter/surface/audio success.
- **PASS / FAIL / UNVERIFIED:** PASS for exact synthetic producer matrix; real OS,
  WGPU and WASM startup failure delivery is UNVERIFIED unless actually exercised.

### F20 — teardown and late callback

- **Fault injected:** destroy tool incarnation, create a new one, then deliver the
  captured old device-lost producer callback.
- **Expected observable facts:** ended/stale old incarnation, distinct current
  incarnation and late event attributed only to old source.
- **Expected unavailable facts:** real backend teardown ordering.
- **Expected process outcome:** normal probe exit.
- **Expected diagnostic outcome:** old attribution or drop, never role reuse.
- **Allowed information loss:** old record may be evicted under capacity pressure.
- **Forbidden effects:** use-after-free, ID reuse or attribution to current.
- **PASS / FAIL / UNVERIFIED:** PASS iff synthetic late delivery is attributed to
  old identity; FAIL on reuse/misattribution; real callback timing UNVERIFIED.

### F21 — permission/disk/write failure

- **Fault injected:** consumer writer targets a path that is a directory, causing
  a deterministic write-open error without filling disk or changing permissions.
- **Expected observable facts:** consumer error and unchanged readable capture.
- **Expected unavailable facts:** footer/final file for the failed target.
- **Expected process outcome:** normal controlled exit.
- **Expected diagnostic outcome:** PRESENT failure remains outside GPE.
- **Allowed information loss:** failed consumer artifact.
- **Forbidden effects:** engine panic, retry loop or diagnostic feedback event.
- **PASS / FAIL / UNVERIFIED:** PASS iff write fails promptly and capture/event
  count stays intact; FAIL on feedback/interference; true disk-full is UNVERIFIED.

### F22 — compile absent and runtime off

- **Fault injected:** A/B compile without diagnostics, then diagnostics compiled
  with an explicit disabled handle and historical `run` path checks.
- **Expected observable facts:** compile-absent succeeds; disabled handle reports
  `COLLECTION_DISABLED`/`NOT_APPLICABLE` with no runtime store/events.
- **Expected unavailable facts:** runtime events and full interactive game A/B.
- **Expected process outcome:** normal checks/probe exit.
- **Expected diagnostic outcome:** disabled mode is explicit, not unknown runtime.
- **Allowed information loss:** all runtime diagnostics by design.
- **Forbidden effects:** hidden ring/callbacks or fabricated attempted collection.
- **PASS / FAIL / UNVERIFIED:** PASS only if absent and disabled structural checks
  pass; deterministic full-engine state/performance A/B remains UNVERIFIED unless
  a credible headless workload exists.

## D. Results

| ID | Fault | Platform | Executed | Result | Evidence | Notes |
|---|---|---|---:|---|---|---|
| F01 | main panic unwind | Windows native debug | YES | PASS | exit 101 in 13 ms; successful artifact: payload, Running, primary incarnation | consumer hook read the same store; no degradation |
| F02 | worker panic | Windows native debug | YES | PASS | worker panic observed; main continued; exit 0 in 22 ms | no frame was fabricated |
| F03 | contended L2 section | Windows native debug | YES | PASS | renderer `UNAVAILABLE`, runtime `KNOWN`, read 42 us | below 100 us gate; section-local degradation |
| F04 | formatter error | Windows native debug | YES | PASS | event count 1 before/after; exit 0 | PRESENT did not feed CAPTURE |
| F05 | hook formatter re-panic | Windows native debug | YES | PASS | one guarded attempt; abnormal exit `-1073740791` in 14 ms | partial report expected; no loop/hang |
| F06 | panic=abort | Windows native debug, abort build | YES | PASS | artifact says `PanicStrategy::Abort`; abnormal exit `-1073740791` in 11 ms | hook ran; no unwind claim; footer occurrence is not generalized as guaranteed |
| F07 | `process::abort` | Windows native debug | YES | PASS | pre-abort breadcrumb; abnormal exit `-1073740791` in 14 ms | partial/no final report correctly accepted |
| F08 | external termination | Windows native debug | YES | PASS | readiness breadcrumb, exact-child kill, exit 1 in 12 ms | no final report expected |
| F09 | hang | Windows native debug | YES | PASS | heartbeat; timeout at 1,016 ms; exact-child kill | experimental L3 observation, not product L2 |
| F10 | deliberate deadlock | Windows native debug | YES | PASS | pre-deadlock heartbeat; timeout at 1,021 ms; exact-child kill | parent remained live |
| F11 | allocator failpoint | Windows native debug | YES | PASS | materialization succeeded while allocations were rejected | current materializer required no allocation; real host OOM unverified |
| F12 | synthetic WGPU Validation | Windows native debug | YES | PASS | exact Validation + primary incarnation | synthetic path verified; real backend delivery unverified |
| F13 | simulated WGPU OOM | Windows native debug | PARTIAL | UNVERIFIED | no contract-compatible OOM producer exists | no real GPU allocation and no uncaptured handler; no certainty fabricated |
| F14 | simulated device lost | Windows native debug | YES | PASS | old incarnation 1; 192-byte truncated message | synthetic path verified; real callback unverified |
| F15 | Lost then Outdated | Windows native debug | YES | PASS | events `Lost, Outdated`; scalar `Outdated`; incarnation 1 | categories remained distinct |
| F16 | synthetic audio error | Windows native debug | YES | PASS | Native/Stream, no frame association | synthetic producer verified; real CPAL callback unverified |
| F17 | saturation/long run | Windows native debug | YES | PASS | 1,000,000 events; 64 retained; 999,936 overwritten; 0 allocations | store 31,272 B; observation 32,008 B; counter saturated without wrap |
| F18 | concurrent producers | Windows native debug | YES | PASS | 100,000 attempted = 64 retained + 82,444 overwritten + 17,492 dropped | 0 torn records; no causal-order claim |
| F19 | startup stage failures | Windows native debug | PARTIAL | UNVERIFIED | synthetic create-surface/request-adapter/request-device matrix truthful | real OS/WGPU/WASM stages and every subsystem stage not injected |
| F20 | teardown/late callback | Windows native debug | PARTIAL | UNVERIFIED | synthetic callback remained on old incarnation 1; new incarnation 2 stayed Unknown | real backend teardown timing/failure not triggered |
| F21 | consumer writer failure | Windows native debug | YES | PASS | deterministic access-denied error; events 1 before/after; 0 retries | real disk-full unverified and intentionally not attempted |
| F22 | compile absent/runtime off | native + WASM compile controls | PARTIAL | UNVERIFIED | default build PASS; disabled handle 792 B and all runtime sections N/A; four WASM example checks PASS | no credible full-engine deterministic A/B workload, so functional/performance equality remains unverified |

Counts: **18 PASS, 0 FAIL, 4 UNVERIFIED, 0 BLOCKED**. `NOT APPLICABLE`
was not used as a whole-scenario verdict; it appears only as the correct field
availability in F22 runtime-off observations.

Evidence directories (not source-controlled) are:

- `target/diagnostics-fault-campaign/native-debug/` for F01-F22;
- `target/diagnostics-fault-campaign/panic-abort/` for the authoritative F06 run.

The initial default-unwind F06 smoke observation is retained but is not the F06
verdict evidence; the separately built abort artifact is authoritative.

## E. Non-interference findings

- **Drop rather than wait:** F03 returned in 42 us with only the contended
  renderer section unavailable. F18 completed all producers and accounted for
  17,492 contention drops instead of waiting globally.
- **Bounded rather than exhaustive:** F17 retained exactly 64 records after one
  million inserts and reported exactly 999,936 overwrites. Store and observation
  physical sizes remained 31,272 B and 32,008 B.
- **Allocation:** F17 observed zero allocation across its million CAPTURE calls
  and materialization. F11's materialization completed while the harness
  allocator rejected every allocation attempt.
- **Counters:** the boundary-seeded counter stayed at `u64::MAX`, set
  `saturated=true`, did not wrap and did not panic.
- **Text:** F14 copied a 768-byte synthetic device-lost message into exactly 192
  bytes with `truncated=true`.
- **PRESENT separation:** F04 formatter failure and F21 writer-open failure did
  not change event counts, create recursive events or alter engine decisions.
- **Termination latency:** destructive children terminated or were cleaned up in
  11-22 ms except the intentionally timed F09/F10 at approximately one second.
- **Observer limitations:** no credible RSS plateau or full-game deterministic
  A/B was available. Fixed layout and allocation counts are strong boundedness
  evidence, but are not mislabeled as full host-memory/performance equivalence.

## F. Unexpected findings

1. On this Windows host, both a panic while processing a panic (F05) and
   `process::abort` (F07) produced status `-1073740791`. Exit status alone cannot
   distinguish these termination causes; stderr, scenario identity and prior
   artifact are necessary. The harness therefore does not invent a portable
   signal/exception taxonomy.
2. F18 intentionally demonstrated substantial diagnostic loss under extreme
   contention (17,492/100,000 publications), while accounting remained exact and
   the engine-facing producers completed. This is expected fail-open behavior,
   not a hidden PASS obtained by requiring exhaustive capture.
3. F06 happened to write a complete footer before abort on this run. The contract
   does not guarantee that delivery, and the result is recorded as one observed
   occurrence rather than a new promise.

## G. UNVERIFIED / BLOCKED

Exhaustive `UNVERIFIED` list:

- F13 real or synthetic uncaptured WGPU OOM category under frozen Option A;
- F19 real platform/backend failures at every startup step;
- F20 real backend teardown failure and real late callback scheduling;
- F22 deterministic full-engine A/B state, framebuffer, CPU and runtime memory;
- real WGPU validation/device-lost callback delivery and callback thread context
  beyond the explicitly verified synthetic F12/F14 producer paths;
- real CPAL callback/backend/device variability beyond synthetic F16;
- real host OOM/OOM killer, real GPU exhaustion, real driver/device reset, native
  access violation, real disk-full and RSS-level plateau;
- browser runtime fault injection/timing/allocation; WASM was compile-checked only;
- interactive primary/tool renderer fault delivery on a real window/GPU.

No scenario is `BLOCKED`. Unsafe host-wide experiments were deliberately not
substituted with simulations and then called verified.

## H. Contract impact

**none.** No executed experiment falsified a frozen clause. The four unverified
scenarios identify missing environmental evidence, not a contradiction or an
impracticable contract requirement. No basis exists to reopen the FROZEN
contract.

## I. Final verdict

**PASS WITH CONDITIONS**

All safely executable process/store oracles passed. Important real backend,
teardown and full-engine A/B properties remain unverified, so `PASS` would
overstate the evidence. There is no demonstrated violation requiring `FAIL` and
no evidence requiring `CONTRACT REVISION REQUIRED`.

### Mandatory closing questions

1. **Did diagnostics aggravate an injected fault?** No demonstrated case. F05's
   abort was injected in the consumer hook itself and did not originate in GPE.
2. **Did an L2 read wait when it should abandon?** No; F03 returned in 42 us with
   the contended section unavailable.
3. **Was unavailable data presented as known?** No demonstrated case.
4. **Was stale data presented as current?** No demonstrated case.
5. **Was a late event attributed to the wrong renderer incarnation?** No; F20
   kept it on old incarnation 1 and left incarnation 2 device-lost Unknown.
6. **Did a bounded structure grow without bound?** No demonstrated case; F17
   plateaued at 64 fixed records with zero measured allocations.
7. **Did saturation cause wrap, corruption or false state?** No.
8. **Did a callback path call a backend or perform forbidden work?** No in the
   synthetic paths; real backend callback contexts remain unverified.
9. **Did any experiment modify fatal WGPU policy?** No.
10. **Was an L3 scenario falsely presented as covered by L2?** No; F08-F10 use an
    experimental parent supervisor and are labeled as such.
11. **Was a legitimate NO FINAL REPORT treated as failure?** No; partial reports
    passed F05 and F07-F10 according to their preregistered oracles.
12. **Is there experimental proof requiring the FROZEN contract to reopen?** No.

## J. Recommended next step

Human review of this campaign, followed by a separately authorized environment
campaign for real WGPU/device-lost, CPAL, teardown, browser and deterministic A/B
coverage. If reviewers want F13 to become executable, they must first decide how
to reconcile that request with frozen WGPU Option A; Mission 3 must not install
an uncaptured handler to manufacture coverage.

No production correction is recommended from the executed evidence. Do not
start Mission 4 or an L3 product architecture from this report.

## K. Final validation and preservation record

Final validations on the reported worktree:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo test` | PASS — 275 tests |
| `cargo test --all-features` | PASS — 293 tests, 1 ignored manual timing probe |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `git diff --check` | PASS |
| `cargo check` | PASS — compile-time diagnostics absent control |
| `cargo check --features diagnostics` | PASS |
| `cargo check --target wasm32-unknown-unknown --example snake_web` | PASS |
| same `snake_web` with `--features diagnostics` | PASS |
| `cargo check --target wasm32-unknown-unknown --example arcade_web` | PASS |
| same `arcade_web` with `--features diagnostics` | PASS |

Final SHA-256 values are unchanged:

- adversarial review:
  `b764a13a8d83e21cf03b4ac1ea41a40ef7572f9593dbbc4e603985688a222040`;
- frozen architecture contract:
  `dad71c2d3b0beef07ff42d8e5ac0863d30b3193813f9c10ed519c95815659f55`.

Files authored or modified by Mission 3 are `Cargo.toml`, `src/lib.rs`,
`src/diagnostics.rs`, `src/bin/diagnostics_fault_probe.rs` and this report.
There is no production fault correction. The pre-existing workspace instruction
files `AGENTS.md` and `RTK.md` are untracked and excluded from Mission 3.

No commit was created; the optional commit field is therefore **none**.

# STOP
