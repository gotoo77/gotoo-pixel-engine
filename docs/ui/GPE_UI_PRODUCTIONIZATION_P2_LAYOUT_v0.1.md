# GPE.UI — PRODUCTIONIZATION P2 / CONSUMER-GRADE LAYOUT COMPOSITION v0.1

Status: **IMPLEMENTATION COMPLETE — HUMAN RUNTIME GATE PENDING**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P1 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P1_RESULT_v0.1.md`

Baseline entering P2:

`2e8650c205827c883e0590b6d655958ca5fe3a27`

Validated implementation head:

`2170c3c2656fb247ae682f439986c34e2ddfb104`

Automated validation:

`CI #668` — **PASS**

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

`experimental_spatial::GridSpec` may remain as a compatibility alias/wrapper during P2.

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

P2 makes responsive Grid a generic layout capability rather than a Card-owned algorithm.

Implemented direction:

```text
UiBuilder
    Column
    Panel
    Grid(UiGridSpec)
```

Grid children are arbitrary transient UI nodes. Grid layout does not know about cards.

The existing `experimental_spatial::run_card_grid*` calls remain compatibility adapters during P2 and preserve MFE-001B behavior.

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

P2 does not change platform/input transport semantics.

---

# 8. Implementation sequence

## P2.1 — shared integer layout primitives — PASS

- introduced `src/ui/layout.rs`;
- canonical `UiGridSpec` and generic `UiGridLayout`;
- deterministic inset and responsive grid track geometry;
- exact tests for 1/2/3 columns, remainder distribution, zero/unusable geometry and saturating inset.

## P2.2 — Spatial adapter convergence — PASS

- Spatial card cells are resolved through shared `layout_responsive_grid`;
- Card-specific image/text subdivision remains in `experimental_spatial`;
- independent Card-owned track computation is removed;
- accepted MFE-001B behavior remains covered.

## P2.3 — transactional builder Grid — PASS

- `UiBuilder::grid(UiGridSpec, ...)` added to the transient graph;
- arbitrary widgets compose inside Grid;
- transactional Grid uses the same shared responsive grid geometry as Spatial;
- headless responsive composition test added.

## P2.4 — linear convergence — PASS

- shared `layout_vertical_children` owns final vertical child placement;
- Root/Column/Panel arrangement routes through that primitive;
- measurement and transactional behavior remain unchanged in intent and covered by the existing test suite.

## P2.5 — closure review — PASS

Proven on implementation head `2170c3c2656fb247ae682f439986c34e2ddfb104`:

```text
one generic responsive grid algorithm      = layout::layout_responsive_grid
one generic vertical child-placement algo  = layout::layout_vertical_children
Spatial Grid uses shared resolved cells     = YES
transactional Grid uses shared resolved cells = YES
Card-specific track computation remains    = NO
styling scope creep                         = NO
input/platform scope creep                  = NO
```

PR #79 changed-file boundary at closure review:

```text
docs/ui/GPE_UI_PRODUCTIONIZATION_P2_LAYOUT_v0.1.md
src/ui/experimental.rs
src/ui/experimental_spatial.rs
src/ui/layout.rs
src/ui/mod.rs
```

---

# 9. Acceptance gates

Automated gates on CI #668 / implementation head `2170c3c2656fb247ae682f439986c34e2ddfb104`:

```text
cargo fmt --check                                      PASS
cargo test                                             PASS
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release --examples                      PASS
Web build/package CI                                   PASS
Conventional Commits CI                               PASS
```

Headless behavioral gates:

```text
zero-size geometry deterministic                       PASS
padding/gap saturating arithmetic deterministic        PASS
responsive 1/2/3-column behavior preserved            PASS
remainder-pixel distribution preserved                 PASS
arbitrary transactional widgets can live in Grid      PASS
nested Column/Panel geometry preserved by regression suite
spatial focus ranking consumes final resolved geometry PASS
pointer/touch behavior unchanged by regression suite  PASS
consumer build closure still runs once                PASS
```

Human runtime:

**PENDING.** A final Native smoke test of the existing MFE-001B probe is required because P2 changes resolved geometry.

A browser runtime rerun is not required: P2 did not change platform/render/input transport semantics; Web build/package is PASS.

---

# 10. STOP

Current state:

```text
P2 IMPLEMENTATION = COMPLETE
P2 AUTOMATED GATES = PASS
P2 CLOSURE REVIEW = PASS
P2 HUMAN NATIVE RUNTIME = PENDING
STOP BEFORE MERGE
```

Final verdict is intentionally withheld until the human Native gate:

```text
GPE.UI PRODUCTIONIZATION P2 = PENDING HUMAN RUNTIME
```
