# GPE.UI — PRODUCTIONIZATION P2 / CONSUMER-GRADE LAYOUT COMPOSITION v0.1

Status: **ACTIVE IMPLEMENTATION CONTRACT — P2 ONLY**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P1 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P1_RESULT_v0.1.md`

Baseline entering P2:

`2e8650c205827c883e0590b6d655958ca5fe3a27`

---

# 1. Mission

Converge the accepted linear/Tiny and responsive spatial/Grid geometry paths around one reusable integer/pixel-aware layout layer, without expanding into styling or a browser-like layout system.

P1 unified interaction. P2 addresses the remaining layout split:

```text
transactional/Tiny
    measure_node / arrange_node
    Root + Column + Panel vertical arrangement

spatial/Grid
    GridSpec
    layout_cards
    CardLayout geometry
```

The production endpoint must not retain Grid geometry as a Card-only subsystem.

---

# 2. Required end state

A shared layout layer owns generic geometry rules used by both adapters:

```text
integer Rect / Size / Constraints
inset / padding
gap handling
vertical linear arrangement
responsive grid track computation
stable remainder distribution
zero-size / overflow-safe arithmetic
```

The shared layer must remain independent of:

```text
Card
Image
TextRenderer
ActionId
interaction state
painting
consumer game state
```

Card-specific image/text subdivision may remain in the compatibility adapter after the generic card cell Rect has been resolved.

---

# 3. Generic grid contract

The MFE-001B responsive behavior is accepted evidence and must remain exact.

Canonical generic grid parameters:

```rust
UiGridSpec {
    min_cell_width: u32,
    preferred_cell_height: u32,
    gap: u32,
    padding: u32,
}
```

Required deterministic behavior:

```text
column count responds to available width
at least one column when usable space exists
columns never exceed child count
remainder pixels are distributed deterministically from the first track
row count = ceil(child_count / columns)
cell height is preferred height capped by available fit
final geometry uses integer pixels only
```

`experimental_spatial::GridSpec` may remain as a compatibility alias during P2.

---

# 4. Linear composition contract

The accepted transactional path currently proves vertical composition only.

P2 therefore productionizes the proven primitive first:

```text
Vertical / Column
```

It must support:

```text
padding
gap
child measured heights
integer final Rects
nested containers
```

P2 MUST NOT add speculative Row/Flex/Stack/absolute/scroll APIs merely for completeness. Those require separate evidence or a later slice.

Panel remains a semantic/painted container that may use the shared vertical layout primitive with padding.

---

# 5. Builder integration

P2 should make responsive Grid a generic layout capability rather than a Card-owned algorithm.

Target direction:

```text
UiBuilder
    Column
    Panel
    Grid(UiGridSpec)
```

Grid children are arbitrary transient UI nodes. Grid layout must not know about cards.

The existing `experimental_spatial::run_card_grid*` calls remain compatibility adapters during P2 and must preserve MFE-001B behavior.

No public v1 naming/API freeze is authorized by this slice.

---

# 6. Compatibility requirements

Existing calls remain valid during P2:

```text
experimental::run
experimental::run_with_input
experimental::run_headless
experimental::run_headless_with_input
experimental_spatial::run_card_grid
experimental_spatial::run_card_grid_headless
```

Accepted behavior to preserve:

```text
Tiny deterministic vertical geometry
Tiny single-build transaction semantics
Grid wide / medium / small column counts
Grid stable remainder distribution
Grid focus/filter/reorder behavior
Grid pointer/touch interaction from P1
custom CardPainter escape hatch
headless deterministic dumps/tests
```

---

# 7. Scope boundaries

P2 MUST NOT implement:

```text
styling / state visuals / rounded corners -> P3
typography changes                      -> P4
debug allocation optimization           -> P5
consumer migration                      -> P6
public v1 API freeze                     -> P7
markup / SVG / DOM / CSS
Taffy integration without new evidence
animation
legacy toolkit removal
```

P2 must not change platform/input transport semantics.

---

# 8. Implementation sequence

## P2.1 — shared integer layout primitives

- introduce a layout-focused module/primitive layer;
- move generic inset, vertical arrangement and responsive grid geometry there;
- add exact headless unit tests for edge cases and MFE-001B track behavior.

## P2.2 — Spatial adapter convergence

- make `GridSpec` a compatibility alias/wrapper over the shared grid spec;
- replace Card-owned grid track computation with the shared layout layer;
- keep Card-specific image/text subdivision and painter behavior outside the generic layer;
- preserve all MFE-001B tests.

## P2.3 — transactional builder Grid

- add generic Grid layout node support to the transient transactional graph;
- use the same shared responsive grid geometry;
- preserve existing Column/Panel behavior;
- add headless tests proving arbitrary widgets can compose in responsive Grid.

## P2.4 — linear convergence

- route Root/Column/Panel vertical final geometry through the shared linear primitive;
- remove duplicate vertical placement arithmetic where safely possible;
- preserve measurement and all transactional behavior.

## P2.5 — closure review

Prove:

```text
one generic responsive grid algorithm
one generic vertical child-placement algorithm
no Card-specific track computation remains
no styling/input/platform scope creep
```

---

# 9. Acceptance gates

Automated:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --examples
Web build/package CI
Conventional Commits CI
```

Headless behavioral gates:

```text
zero-size geometry deterministic
padding/gap saturating arithmetic deterministic
responsive 1/2/3-column behavior preserved
remainder-pixel distribution preserved
arbitrary transactional widgets can live in Grid
nested Column/Panel geometry preserved
spatial focus ranking still consumes final resolved geometry
pointer/touch behavior unchanged
consumer build closure still runs once
```

Human runtime:

A final Native smoke test of the existing MFE-001B probe is required because P2 changes resolved geometry.

A browser runtime rerun is required only if platform/render/input transport semantics change. P2 must not change them; Web build/package is otherwise sufficient.

---

# 10. STOP

When P2 passes:

```text
GPE.UI PRODUCTIONIZATION P2 = PASS / PASS WITH CONDITIONS / FAIL
STOP
```

Record a formal P2 result before starting P3.
