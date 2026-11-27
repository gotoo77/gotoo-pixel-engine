# Resolved discrete controls (#117)

Correction contract: [v2](https://github.com/gotoo77/gotoo-pixel-engine/issues/117#issuecomment-7229097296).
Correction base: `cd515837939126f5bf430436338e30beab7f0a6f`.

`ui::resolved` is a thin adapter over the existing interaction pass with spatial
navigation. The caller owns layout and painting. There is one interaction state,
owned by the kernel; the adapter adds validation, input arbitration and a neutral
snapshot barrier. Existing adapters retain their policies; only this adapter
selects the shared kernel cone preference.

```rust
use gotoo_pixel_engine::{ActionId, Rect, ui::resolved::{self, ResolvedState,
    ResolvedTarget, UiId, UiInput}};

let mut state = ResolvedState::default();
let controls = [ResolvedTarget {
    id: UiId::from_stable(1).unwrap(),
    rect: Rect { x: 30, y: 20, width: 80, height: 35 },
    action: ActionId::new("select"),
    enabled: true,
}];
let output = resolved::update(&mut state, UiInput::default(), &controls)
    .expect("valid resolved geometry");
// Paint controls[i].rect; use output.focused, hovered and active(id).
// Dispatch output.activation (one ID/action pair at most) explicitly.
```

Each ID must be unique per surface, non-ROOT and stable across reordering and
layout changes. `UiId::from_stable` rejects the reserved root value. Do not reuse
an ID for another action during a gesture. No global registry is introduced.

All rectangles, including disabled targets, must be nonempty and disjoint. They
use half-open edges; adjacent edges may touch. Validation uses existing i64
rectangle intersection arithmetic, including at i32/u32 extremes. The caller
supplies geometry visible in the framebuffer; `update` has no framebuffer size
and does not clip. Decorations are not targets. Overlays, overlapping controls,
z-order, continuous controls and general widgets are outside this API.

An invalid frame returns `ResolvedError`, delivers no result/action, clears all
captures and requires a neutral snapshot before interaction resumes. Disabled
targets remain available to the painter but are filtered before the kernel.
Removal/disabling uses the kernel's existing focus and capture repair, including
its previous-index fallback. Reinserted/re-enabled IDs cannot resume old captures.
Spatial candidates must lie in the requested open half-plane. Candidates inside
the 45-degree cone (primary >= secondary, boundary included) rank before those
outside it; primary distance, secondary distance and slice order break ties.
Without an in-cone candidate, the old deterministic ranking applies. Disabled
targets cannot participate. A reorder preserves focus identity but may change
an exact tie. This minimal playtest correction is not universal ergonomic proof,
and does not guarantee reciprocal navigation: in the probe, Right then Left
selects Up, which is in-cone and 2px nearer on the primary axis than Center.
Historical Spatial/Linear adapters retain their original behavior.

Cancel suppresses navigation, captures and activation. Any directional input,
pointer edge, touch event, ongoing capture, or loss of the previous focused
target suppresses concurrent confirm. Pointer events still use the kernel's
press/release semantics: release activates only the captured target under the
pointer; touch move-out/cancel drops that touch capture. If distinct pointer IDs
activate in one frame, the adapter returns no activation. Multiple sources on the
same ID are deduplicated by the kernel. Navigation alone never activates.

`active(id)` means actual mouse/touch capture, including a mouse held outside its
rectangle until release. Keyboard/pad snapshots have no held-confirm state. Show
focus for keyboard/pad and a short confirmation flash when an activation arrives.
Availability is `ResolvedTarget::enabled`, not retained visual state.

Call `state.suspend()` **before** the frame to withhold. Captures clear immediately;
the next update preserves valid focus through the kernel and withholds events
until it has consumed a neutral semantic snapshot (no navigation, pointer edges
or touch events; a hover position is neutral). The neutral frame cannot activate.
Repeated suspension remains blocked. Pending release/end events cannot activate.
Snapshots contain events rather than physical held state; a caller needing a
physical release barrier must gate its input snapshots as well.

For device snapshots, call `state.synchronize_input(frame.input)` before handling
semantic events **on every update**, including unfocused updates. It bridges the
platform lifecycle into `suspend()` and returns whether caller-owned transitions
may run. On false, clear confirmation flashes and suppress toggles/other actions;
still call `resolved::update` with the semantic snapshot so the neutral barrier
can advance. The probe demonstrates this order with `ControlMap`.

`Input::window_focused()` is the last real focus state; the platform initializes
it from the main/tool window and forwards both gain and loss. A durable
`window_reset_generation()` detects even loss+gain between game updates. Loss
clears window devices exactly as before, with no invented release/end. Gain does
not synthesize held state. The generation survives `advance_frame` and cloning;
`PartialEq` includes both lifecycle values. Standalone `Input::default()` starts
focused with generation zero. Snapshot producers use `set_window_focused` for
actual notifications; hosted consumers forward or clone the original snapshot.
The current `PixelGameHost` forwards it by reference, retaining actual lifecycle.
Pointer position None and empty touch frames are never focus-loss signals.

Reentry requires real focus, no observable key/mouse/gamepad button held or edge,
no touch events and zero gamepad axes, followed by a neutral semantic snapshot.
Gamepads survive the window reset, so held buttons/axes continue blocking even
on frames without pressed events. All axes conservatively require zero; drift
can delay reentry. Physical release of window keys/mouse/touches held outside
the window cannot be proven after their state was erased by the OS reset.
Sparse touch frames are not proof of physical release. Inherited release/end
cannot activate because the capture is gone; a fresh press/start is required.
Semantic-only callers retain the explicit `suspend()` contract. Switching input
sources for the same state requires explicit suspension as well.

Runtime remains HOLD until real native OS focus and Web retests. This work does
not authorize Pirates integration or S2.

## Example and public tests

```sh
cargo run --locked --example gpe_ui_resolved
cargo run --locked --example gpe_ui_resolved --features outline-fonts
cargo test --locked --test ui_resolved
```

The example uses five disjoint irregular rectangles and paints exactly those
rectangles. Arrow keys / pad D-pad navigate; Enter/Space / South confirm;
Escape / East cancel; R / Start explicitly suspend; D / West toggles the bottom
control. Cyan border: focus. Blue fill: hover. Green fill: capture. Gray:
unavailable. Orange flash: confirmed activation. The optional label font is an
existing bundled font; no assets or dependencies were added. The example maps
public device snapshots through `ControlMap` to `UiNavInput` and supplies public
mouse/touch snapshots. Semantic integration tests do not qualify real hardware.

For a Web run, build the same example for `wasm32-unknown-unknown`, then pass
`target/wasm32-unknown-unknown/debug/examples/gpe_ui_resolved.wasm` to the repository's
matching `wasm-bindgen --target web`. Its exported start hook launches the same
example without a Cargo declaration or Web-host change.

The external tests exercise four-direction navigation, ranking/ties, reordering,
moving geometry, painting/hit-test boundaries, safe extremes, invalid/disabled
geometry, focus fallback, mouse/touch captures, cancel/confirm/nav conflicts,
simultaneous sources, ambiguous multi-touch and explicit reset/reentry.

## Qualification

See `RESOLVED_TARGETS_117_V2_VALIDATION.md` for correction evidence and retests.
`RESOLVED_TARGETS_117_VALIDATION.md` records the historical v1 qualification.
