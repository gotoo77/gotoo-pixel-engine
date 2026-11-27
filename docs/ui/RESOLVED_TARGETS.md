# Resolved discrete controls (#117)

Implementation contract: [v1](https://github.com/gotoo77/gotoo-pixel-engine/issues/117#issuecomment-7228671898).
Reference: `0b98ebb58f92d8134f9308b106a221a47b7655cf`.

`ui::resolved` is a thin adapter over the existing interaction pass with spatial
navigation. The caller owns layout and painting. There is one interaction state,
owned by the kernel; the adapter adds validation, input arbitration and a neutral
snapshot barrier. Existing adapters and the kernel are unchanged.

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
Spatial candidates retain primary-distance, secondary-distance and slice-order
tie-breaking. A reorder preserves focus identity but may change an exact tie.

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

There is no automatic OS-focus notification in this adapter. The platform's
`Focused(false)` resets window devices, but that does not notify `ResolvedState`.
Explicit reset tests do not qualify real OS focus loss/resume. Input/platform
changes require a separate ticket. Real OS focus and browser resize qualification
remain necessary before returning to Pirates; this work does not authorize S2.

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

See `RESOLVED_TARGETS_117_VALIDATION.md` for commands, results and limitations.
