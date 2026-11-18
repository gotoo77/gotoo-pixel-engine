# B3: experimental per-loop playback rate

2026-11-18 — **GO WITH CONDITIONS for experimental review only**. No merge,
SMOG CITY integration, human listening or H1 validation is implied.

## Contract

`Audio::set_loop_playback_rate(&mut self, playback: PlaybackId, rate: f32) -> Result<(), AudioError>`
changes one existing loop, initially at 1.0. Rates must be finite and within
**0.5..=2.0**, inclusive. Invalid rates and unknown, stopped or file-playback IDs
fail before mutation. IDs belong to their Audio instance. Speed changes pitch
and cycle duration together. No player/source restart, new ID, individual gain,
bus/master change or dependency update is introduced. Consumers own smoothing.

- Native reuses the retained Rodio Player. Private `LoopBuffer` reads shared,
  validated PCM in real **1024-frame** spans (1024 mono / 2048 stereo samples),
  including a frame-aligned tail, followed by the existing `repeat_infinite()`.
- Web writes the retained source's `playback_rate()`. Active IDs are published
  after connect/start succeeds; failed/stopped nodes are disconnected. A closed
  context rejects changes; suspended contexts can retain the rate.
- Noop remembers the rate until stop. Logical native loops without a device
  behave deterministically too. Success does not prove an audible device.
- Repeated equal rates avoid native/Web control writes. Custom Audio backends
  retain a default unsupported error. Runtime native device loss uses the
  existing asynchronous error callback. Reset means stop/discard IDs and start
  fresh loops; there is no new reset API.

## Baseline and dependencies

Actual remote main and the consumed Cargo checkout are both
`8cbceda0c9b0cc55ee5deb7d5e173676ffc1d4f1`; Audio and GPE's Cargo.lock matched.
The unrelated development worktree remains on `game/void-canticle`,
`39eec6bc0ebe0f3e6d323154014bc6c8a30b5b69`. The dedicated branch starts from main.
Rust **1.97.1**, Rodio **0.22.2**, CPAL **0.17.3**, web-sys **0.3.104** were used.
Cargo.lock is unchanged.

Exact installed Rodio root:
`/home/gduval/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rodio-0.22.2/src/`.
`source/mod.rs:181` documents total span size, rather than remaining samples;
`source/uniform.rs:55` caps chunks at 32768 samples; `player.rs:130` uses 5 ms
periodic control access; `player.rs:204` exposes set_speed. `source/repeat.rs`
and `source/buffered.rs` cache/extract actual spans. PCM tests check exact order
over five mono/stereo cycles and partial tails, without claiming transparent resampling.

## Actual GPE capture

The ignored `audio::native::rate_probe::capture` test creates real NativeAudio
with no Noop fallback. One target on Sfx receives all rate changes; a constant
antiphase 4 kHz sentinel on Music remains at 1.0. Assertions check a stable
PlaybackId/player pointer, one target append, untouched gains/buses and stop
cleanup. No per-sample counter instruments the target.

WSLg PulseAudio 17 capture isolates the process's sink-input:

```text
parec --device=RDPSink.monitor --monitor-stream=INDEX --raw --format=float32le --rate=44100 --channels=2 --latency-msec=5 --process-time-msec=2
```

A 128-frame antiphase marker on the same real device mixer records render frame
and wall time. Channel difference locates markers; channel mean isolates the
target. Capture/native-frame synchronization residual was below 0.001 ms.
An original 880 Hz, 1 s PCM16 sine loop follows 1 → 1.5 → 0.75 → 1 four times,
with varied 1.65–2.02 s intervals. Detection fits known frequencies over 8 ms
windows at 1 ms hops, brackets old/new stable windows, and requires eight
consecutive fits. Processing estimate = change relative to captured marker +
command-to-marker render time. The conservative **±6 ms** candidate allowance
covers the detection bracket, timestamp granularity and observed clock drift;
it is an experimental estimate, not a universal hard bound. Reference stereo
clock-offset drift was 2.8 ms; its rounded allowance is less certain, without
affecting the failure conclusion.

Command-to-read also includes mixing/output/capture/scheduling. Those components
cannot be individually separated here. Physical speaker latency is unmeasured.
The reference uses a pristine main snapshot: only its test helper calls private
Player::set_speed because the original public trait lacks it. Its real start_loop
retains SamplesBuffer.repeat_infinite. Candidate calls the new public method.

| Source | N | Median ms | Max ms | Allowance | >100 incl. allowance | Command→read max ms |
|---|---:|---:|---:|---:|---:|---:|
| reference-mono | 12 | 186.9 | 317.2 | ±6 ms | 11/12 | 382.6 |
| reference-stereo | 12 | 149.8 | 358.8 | ±6 ms | 8/12 | 427.5 |
| candidate-mono | 12 | 25.3 | 52.7 | ±6 ms | 0/12 | 121.4 |
| candidate-stereo | 12 | 25.1 | 42.3 | ±6 ms | 0/12 | 112.3 |

Individual processing estimates, in command order (ms):

| Run | Values |
|---|---|
| reference-mono | 124.6, 289.6, 244.7, 229.0, 317.2, 58.0, 112.3, 231.1, 140.0, 144.9, 268.3, 98.4 |
| reference-stereo | 134.3, 51.3, 159.7, 144.0, 155.5, 48.8, 358.8, 235.0, 287.8, 302.2, 91.3, 84.6 |
| candidate-mono | 37.5, 17.3, 32.3, 40.0, 21.6, 26.6, 22.4, 24.0, 14.9, 52.7, 28.5, 20.5 |
| candidate-stereo | 20.8, 19.6, 34.0, 39.6, 18.4, 39.8, 24.1, 25.1, 21.5, 30.2, 42.3, 25.0 |


All 24 candidate changes meet the indicative processing target: **52.7 + 6 =
58.7 ms** worst observed. This does not establish ≤100 ms end-to-end latency:
command-to-read reaches **121.4 ms mono / 112.3 ms stereo**. Different loop/span
phases are sampled; there is no exhaustive worst-phase guarantee.

## Signal quality

No clipping, silence gap ≥1 ms or transition jump above 1.5 times the faster
uninterrupted sinusoid's adjacent-sample bound plus 0.0002 was detected. Candidate
sentinel amplitude remains **0.00499913–0.00499968**, supporting emitted-signal isolation.

Short spans are **not spectrally transparent**. In 400 ms steady windows, fitted
pitch deviates by up to **0.901 cent** (reference ≤0.081 cent): approximately
**1319.36 Hz** instead of 1320 Hz at 1.5, and **660.16 Hz** instead of 660 Hz at
0.75. Frequency-corrected sine residual varies with phase/window: candidate worst
−28 dB, reference worst −29 dB mono / −34 dB stereo. A representative 1.5 window
is about −35 dB candidate versus −60 dB reference mono. These are measurable
resampling/modulation errors, without a claim of inaudibility. Rodio is unchanged.

A provisional motor WAV was copied outside both repositories from SMOG CITY's
`feat/p1-5-audio-feedback:assets/audio/propulsion.wav` (mono PCM16, 44100 Hz, 1 s).
Six transitions per version show dominant peaks near 36 → 54 → 27 → 36 Hz,
without clipping or long gaps. Low harmonic band power differences for
36/72/108 Hz × rate, ±8 Hz bands, were ≤0.005 dB. The noisier 396 Hz band differs
by up to 2.83 dB; windows are not asset-phase aligned, so spans are not the only
possible cause. Maximum active adjacent-sample jump is 0.008606 on both versions.
The sinusoid latency detector was not applied to the motor. No human listened.

An optional separate CPU attempt could not open the native device, so its cost
numbers are unusable: **B3 CPU/allocations NOT RUN**. B2 isolated cost data is not
substituted for integrated evidence.

## Validation and remaining conditions

Tests cover initial/boundary/invalid rates, unknown/stopped/file IDs, two loops,
gains/mute, no-device state, exact PCM/span order and stop cleanup. Real Chromium
checks source writes/identity/isolation, endpoints 0.5/2, mute persistence,
create/connect/start failures without active-ID leaks, and closed-context cleanup.
This is a browser contract test, not a Web signal-latency measurement.

Native canonical checks, locked check/test, strict Clippy with default features,
WASM build and browser tests pass. Strict all-feature Clippy **fails on pristine
main too**: competing global allocators in the cost probe and diagnostics fault
injection, plus existing font gallery warnings. No unrelated example, feature or
CI rule is changed to conceal this failure. The user explicitly authorized
publication as a draft with only these pre-existing failures documented; this
exception does not change their FAIL status or authorize any merge.


### Reproduced all-feature Clippy failures

The exact command below exits **101 on both main and B3**, with Rust 1.97.1:

```sh
cargo clippy --locked --all-targets --all-features -- -D warnings
```

The main snapshot was verified byte-for-byte against all **310 tracked files**
of `8cbceda0c9b0cc55ee5deb7d5e173676ffc1d4f1`, including Cargo.lock.
A second diagnostic-only run adds `--keep-going --message-format=json` before
`-- -D warnings`, to collect all parallel target failures despite cancellation.
Both sets of error messages, codes, targets and primary file/line/column spans
are **identical (4 diagnostics)**:

| Target / source | Location | Diagnostic on main and B3 |
|---|---|---|
| `gpe_ui_mfe_001c_cost_probe` | `examples/gpe_ui_mfe_001c_cost_probe.rs:39` | `#[global_allocator]` conflicts with allocator in `gotoo_pixel_engine` |
| `gpe_ui_p4_font_gallery_web` | `examples/gpe_ui_p4_font_gallery_web.rs:38:4` | `dead_code`: function `main` is never used |
| `gpe_ui_p4_font_gallery` | `examples/gpe_ui_p4_font_gallery.rs:3:5` | `unused_imports`: `TextInputEvent` |
| `gpe_ui_p4_font_gallery` | `examples/gpe_ui_p4_font_gallery.rs:458:9` | `clippy::collapsible_if` |

No new B3 diagnostic appears. Strict Clippy on all targets with default features
and on the library with all features passes. No existing test body, CI file,
lint rule, allocator or font example was modified.

Local proof files: `/tmp/gpe-b3-evidence/publish-{main,b3}-exact.log`,
`publish-{main,b3}-diagnostics.jsonl`, and `publish-clippy-comparison.json`.
The exception authorizes only a draft PR; the failures remain open and out of scope.

Validation totals: `cargo test --locked`: **386 PASS, 1 ignored**; canonical
`python3 scripts/dev.py check`: **400 PASS, 1 ignored**, fmt, strict default
Clippy and diff checks PASS; Web unit tests **102 PASS**; real Chromium **3 PASS**.
The ignored native device probe was explicitly run via the measured capture protocol.

**GO WITH CONDITIONS** for experimental review only. Before merge, reproduce
Windows-native processing ≤100 ms including uncertainty, measure runtime cost,
listen to the motor for artifacts, and measure browser output if supported.
Windows human listening, physical latency and browser signal latency: **NOT RUN**.
SMOG CITY and its PRs #4/#5 remain untouched. H1 stays open. No automatic merge.

## Local reproduction and artifacts

`/tmp/gpe-b3-evidence/` contains capture.py, analyze.py (B1 detector), quality.py,
comparison.json, and candidate/reference mono/stereo/motor directories with
events.json, capture.json, capture.f32le, quality.json, plus tone results.json
and latency.png. B2 at `/tmp/smog-audio-probe-b2` retains 184 verified manifest
entries. Raw audio/motor files are temporary evidence, not committed assets.

Build `cargo test --locked --lib --no-run`; select the lib-test executable and
invoke `--ignored --exact audio::native::rate_probe::capture --nocapture`.
Set `GPE_RATE_PROBE_COUNT=12`, `GPE_RATE_PROBE_CHANNELS=1` or `2`, and optionally
`GPE_RATE_PROBE_WAV=/tmp/.../propulsion.wav`. Wait for ready JSON, start isolated
monitor capture, send `GO` on stdin, and retain all command/render JSON events.
Missing device/capture must fail, without fallback. The temporary scripts implement
this protocol. Browser: `python3 scripts/dev.py check-web-browser`.

Motor fixture SHA256: `a52bae1da31fcba78811491bd598fc2174254e58ca31dd7b0f3f48c571dbd03a`.
