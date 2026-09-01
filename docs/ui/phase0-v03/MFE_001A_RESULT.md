# GPE.UI MFE-001A — RESULT

Status: **PASS**

Branch:

`feat/gpe-ui-mfe-001a`

Human runtime gate date:

`2026-09-01`

## Scope actually validated

MFE-001A tested the smallest experimental transaction core required before responsive/spatial work:

- transient one-frame UI description/graph;
- explicit `UiStateStore`;
- deterministic `UiId` identity;
- `Panel`, `Column`, `Text`, `Button`, `Toggle<bool>`, `Slider<f32>`;
- semantic navigation input;
- typed value proposals returned through `UiOutput`;
- consumer-owned authoritative settings state;
- same-frame effective widget paint;
- headless dump/tests;
- existing GPE framebuffer/text primitives;
- Native and Web compilation/package CI paths.

Explicitly not validated here:

- responsive Grid/Card UI;
- spatial focus;
- mouse/touch expansion;
- production consumer migration;
- markup/SVG/animation/inspector;
- actual browser runtime;
- typography quality/fallback policy.

---

## Transaction model T1

Accepted model:

```text
consumer snapshot
→ build transient UI
→ measure/layout
→ interaction
→ frame-local effective widget values
→ typed proposed changes
→ paint
→ return UiOutput
→ consumer commits authoritative state
```

No hidden replay of the consumer UI closure occurs.

A slider can therefore paint an effective proposed value in the current frame while other text built from the original consumer snapshot can still represent the pre-commit value until the next frame.

---

## Human runtime verdict

Probe:

`examples/gpe_ui_mfe_001a_probe.rs`

Command used conceptually:

```text
cargo run --release --example gpe_ui_mfe_001a_probe
```

Observed interaction:

- focus navigation worked;
- Toggle worked;
- Slider worked;
- Reset worked;
- authoritative state committed correctly after `run()`;
- persisted transaction witness displayed snapshot/effective-proposed values;
- no perceptible/glitchy one-transaction lag was reported.

Human verdict:

```text
T1 = ACCEPTABLE
```

Therefore the transaction model is accepted for continuation into MFE-001B.

---

## Automated/local validation evidence

Human local validation on Windows reported:

```text
cargo test ui::experimental
12 passed
0 failed

cargo clippy --all-targets --all-features -- -D warnings
PASS

cargo build --release --examples
PASS
```

GitHub Actions CI for the final visual-probe head also passed:

```text
conventional commits  PASS
native validation      PASS
web build/package      PASS
```

Actual browser runtime remains **NOT TESTED** and must not be reported as PASS.

---

## Independent-review conditions closure

### `WidgetRef<T>` lifetime/generation

The experimental implementation currently scopes output queries by transaction generation.

This is deliberately conservative and differs from the earlier paper candidate of a stable cross-transaction query token.

For MFE-001A this is accepted as an implementation boundary rather than treated as a blocker:

- widget identity itself remains stable through `UiId`;
- `WidgetRef<T>` is an output-query capability for one transaction;
- future consumer pressure may justify a stable cross-transaction handle, but MFE-001A does not require one.

This decision is **not** a frozen production API guarantee.

### Widget-kind reuse diagnostics

Interactive kind reuse is detected and repairs/reset state deterministically.

The experimental diagnostic coverage is not yet guaranteed for every non-interactive kind transition (for example an interactive widget replaced by plain Text with the same logical identity).

This is accepted as non-blocking diagnostic debt for MFE-001A because stale persistent interaction state is pruned and focus is repaired. Productionization must revisit uniform kind-compatibility diagnostics before treating the experimental module as stable API.

---

## New follow-up discovered by human runtime

The runtime probe exposed a separate presentation problem:

1. current bitmap text is visibly coarse for a polished GPE.UI target;
2. the em dash used in the probe title is unsupported by the current font path and appears as an ambiguous `?`-like fallback;
3. missing-glyph behavior needs an explicit, deterministic policy and recognizable replacement glyph/picto.

This does **not** invalidate MFE-001A transaction semantics.

It is tracked separately as a typography/glyph-fallback follow-up.

---

## Decision

```text
MFE-001A RESULT:
PASS

T1 TRANSACTION:
ACCEPTED

ARCHITECTURE B:
REMAINS LEADING HYPOTHESIS

MFE-001B:
AUTHORIZED

MFE-001C:
BLOCKED ON MFE-001B

PRODUCTION MIGRATION:
NOT AUTHORIZED BY THIS RESULT

WEB BROWSER RUNTIME:
DEFERRED / NOT TESTED
```

MFE-001B should now test the responsive/spatial layer rather than reopen T1 unless new evidence contradicts this result.
