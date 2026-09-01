# GPE.UI — PRODUCTIONIZATION P2 / CONSUMER-GRADE LAYOUT COMPOSITION v0.1

Status: **COMPLETE — PASS / STOP**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P1 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P1_RESULT_v0.1.md`

P2 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P2_RESULT_v0.1.md`

Baseline entering P2:

`2e8650c205827c883e0590b6d655958ca5fe3a27`

Validated implementation head:

`2170c3c2656fb247ae682f439986c34e2ddfb104`

Automated validation:

`CI #668` — **PASS**

Human validation:

`P2 NATIVE RUNTIME = PASS`

---

# 1. Mission

Converge the accepted linear/Tiny and responsive spatial/Grid geometry paths around one reusable integer/pixel-aware layout layer, without expanding into styling or a browser-like layout system.

P1 unified interaction. P2 closes the remaining proven layout split.

---

# 2. Required end state — ACHIEVED

The shared layout layer owns generic geometry rules used by both adapters:

```text
integer Rect / Size / Constraints
inset / padding
gap handling
vertical linear arrangement
responsive grid track computation
stable remainder distribution
zero-size / overflow-safe arithmetic
```

The shared layer remains independent of:

```text
Card
Image
TextRenderer
ActionId
interaction state
painting
consumer game state
```

Card-specific image/text subdivision remains intentionally in the compatibility adapter after the generic card cell Rect has been resolved.

---

# 3. Generic Grid contract — PASS

Canonical parameters:

```rust
UiGridSpec {
    min_cell_width: u32,
    preferred_cell_height: u32,
    gap: u32,
    padding: u32,
}
```

Validated deterministic behavior:

```text
column count responds to available width
at least one column when usable space exists
columns never exceed child count
remainder pixels distributed deterministically from first track
row count = ceil(child_count / columns)
cell height capped by available fit
integer final geometry
```

The Spatial compatibility Grid delegates to the shared generic Grid resolver.

---

# 4. Linear composition contract — PASS

The proven vertical composition primitive supports:

```text
padding
gap
child measured heights
integer final Rects
nested containers
```

Root/Column/Panel final placement uses `layout_vertical_children`.

No speculative Row/Flex/Stack/absolute/scroll system was added.

---

# 5. Builder integration — PASS

Implemented transactional composition:

```text
UiBuilder
    Column
    Panel
    Grid(UiGridSpec)
```

Grid children are arbitrary transient UI nodes. Grid layout has no Card knowledge.

Existing `experimental_spatial::run_card_grid*` calls remain compatibility adapters and preserve MFE-001B behavior.

No public v1 naming/API freeze is authorized by P2.

---

# 6. Compatibility — PASS

Existing calls remain valid:

```text
experimental::run
experimental::run_with_input
experimental::run_headless
experimental::run_headless_with_input
experimental_spatial::run_card_grid
experimental_spatial::run_card_grid_headless
```

Accepted behavior remains covered:

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

# 7. Scope boundaries — PRESERVED

P2 did not implement:

```text
styling / state visuals / rounded corners -> P3
typography changes                      -> P4
debug allocation optimization           -> P5
consumer migration                      -> P6
public v1 API freeze                     -> P7
markup / SVG / DOM / CSS
Taffy integration
animation
legacy toolkit removal
```

P2 did not change platform/input transport semantics.

---

# 8. Implementation sequence

```text
P2.1 shared integer layout primitives  PASS
P2.2 Spatial adapter convergence       PASS
P2.3 transactional builder Grid        PASS
P2.4 linear convergence                PASS
P2.5 closure review                    PASS
```

Closure proof:

```text
one generic responsive grid algorithm        = layout::layout_responsive_grid
one generic vertical child-placement algo    = layout::layout_vertical_children
Spatial Grid uses shared resolved cells       = YES
transactional Grid uses shared resolved cells = YES
Card-specific track computation remains      = NO
styling scope creep                           = NO
input/platform scope creep                    = NO
```

---

# 9. Acceptance gates

Automated gates on CI #668 / implementation head `2170c3c2656fb247ae682f439986c34e2ddfb104`:

```text
cargo fmt --check                                         PASS
cargo test                                                PASS
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release --examples                         PASS
Web build/package CI                                      PASS
Conventional Commits CI                                  PASS
```

Human runtime:

```text
P2 NATIVE RUNTIME = PASS
```

A browser runtime rerun was not required because P2 did not change platform/render/input transport semantics.

---

# 10. STOP

```text
GPE.UI PRODUCTIONIZATION P2 = PASS
STOP
```

P2 is complete. Do not begin P3 on this branch. Merge PR #79 only after final closing-documentation CI is green, then start P3 separately from the resulting `main` baseline.