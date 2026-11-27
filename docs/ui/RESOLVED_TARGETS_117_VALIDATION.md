# GPE #117 qualification — 2026-11-27

Contract: [v1](https://github.com/gotoo77/gotoo-pixel-engine/issues/117#issuecomment-7228671898).
Base: `0b98ebb58f92d8134f9308b106a221a47b7655cf`.
Branch: `feat/ui-resolved-targets-117`. Rust: `1.97.1`.

Scope: one new adapter, module declaration, stable-ID constructor, public
integration tests, standalone example and API/qualification documentation.
No shared kernel, old adapter, input, platform, presentation, renderer, Cargo,
dependency, Pirates or GPE-Agent changes. No merge or pin change.

## Automated evidence

All commands below ran from this worktree. `rtk` was used as a command proxy.
Long logs and runtime artifacts are local under `/tmp/gpe-117/`, not published.

| Command / check | Result |
| --- | --- |
| Preflight `cargo test --locked --lib` | PASS — 391 tests at the reference production tree |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked --test ui_resolved` | PASS — 18 external-consumer tests |
| `cargo test --locked --test ui_resolved --all-features` | PASS — same 18 tests |
| `python3 scripts/dev.py check` | PASS — 449 tests across 27 suites, including separation, rustfmt, library/binary/example/integration tests, strict `cargo clippy --all-targets -- -D warnings`, and diff check; rerun after final test additions and restored mutations |
| `python3 scripts/dev.py check-web` | PASS — canonical Web demo builds |
| `cargo check --locked --lib --test ui_resolved --example gpe_ui_resolved --features outline-fonts` | PASS |
| Same targeted `check` with `--all-features` | PASS |
| Both targeted `check` commands with `--target wasm32-unknown-unknown` | PASS |
| `cargo clippy --locked --lib --test ui_resolved --example gpe_ui_resolved --features outline-fonts -- -D warnings` | PASS |
| Same targeted Clippy with `--all-features` | PASS |
| Both targeted Clippy commands with `--target wasm32-unknown-unknown` | PASS |
| Targeted WASM Clippy with default features and `-D warnings` | PASS |
| `cargo build --locked --example gpe_ui_resolved --features outline-fonts` | PASS |
| Same build with `--target wasm32-unknown-unknown`, then matching `wasm-bindgen --target web` | PASS |
| `git diff --check` | PASS |

The first public test invocation failed before implementation because the public
module did not exist (`E0432`). After implementation, all 18 tests pass. Two
temporary mutations independently produced assertion failures: bypass cancel's
capture reset, and deliver an arbitrary activation when more than one ID was
activated. Both original source versions were restored in a `finally` block;
the complete canonical gate then passed. These failures are intentional evidence
that the discriminant tests catch behavioral regressions.

A single independent, read-only review examined the adapter, unchanged kernel,
public tests, example and tiny export/constructor diff. It found no concrete
correctness or scope issue. No implementation fanout was used.

## Historical broader failure (#112)

`cargo clippy --locked --all-targets --all-features -- -D warnings`: **FAIL**.
Reproduced before production changes: `#[global_allocator]` in the existing
`gpe_ui_mfe_001c_cost_probe` conflicts with the library diagnostics allocator.
See [#112](https://github.com/gotoo77/gotoo-pixel-engine/issues/112).
The other historical diagnostics listed there are not claimed reproduced here.
No correction, lint suppression, feature removal or gate change was made.
The exact targets added by #117 pass strict Clippy with all features on native
and WASM. The broader failure is not presented as green or resolved by this lot.

## Runtime evidence and limits

- **PASS — native presentation smoke:** the compiled outline-font example ran
  under X11. `xwininfo -root -tree` observed
  `GPE #117 resolved controls | 14.6 ms | 63 FPS`, at 960×600. The existing host
  updates that FPS counter only for `RenderOutcome::Presented`. The harness then
  terminated only its own probe process. An earlier 10-second run ended with the
  deliberate timeout (124), without a panic/error log; it was not used alone as
  proof of rendering.
- **PASS — real browser startup and GPU submission smoke:** Chromium
  `151.0.7922.34`, Playwright `1.62.1`, actual wasm-bindgen/winit example, local
  static page. Visible 960×600 canvas, adapter/device creation, configured surface,
  hundreds of RAF callbacks and queue submissions, no page errors. An instrumented
  run observed 449 RAF callbacks / 447 submits. Keyboard/mouse/reset sequences
  were sent, but their visual outcomes are not claimed verified.
- **HOLD — browser visual qualification:** element captures remained transparent;
  whole-page captures did not show the controls. The unchanged canonical
  `web_demo` reproduced the same capture anomaly (442 RAF / 440 submits).
  Two harness adjustments were tried: capture the entire page, then explicit
  ANGLE SwiftShader. Neither qualified the visual output. Investigation stopped;
  no host/renderer change and no assertion that this is a #117 regression or a
  known root cause. A destroyed-device observation was also recorded by the
  temporary tracer; it is not interpreted as proof of successful presentation.
- **NOT RUN — physical keyboard/gamepad/touch qualification and real OS focus
  loss/resume.** Public semantic snapshots, not physical devices, are covered by
  the automated interaction tests. Explicit suspension has no OS focus hook.
- **NOT RUN — Pirates browser resize/clipping and S1-B/S2 integration.** Pirates
  was not modified. Engine tests/builds and native presentation do not grant GO S2.

The #117 automated gates and native presentation qualify this bounded adapter;
browser visuals and real focus/hardware remain explicitly unqualified. Before
returning to Pirates, perform the separate real-browser/focus qualification.
Credits/cost: not measured.
