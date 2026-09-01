# GPE.UI — Text Rendering / Missing Glyph Follow-up

Status: **OPEN FOLLOW-UP — DOES NOT BLOCK MFE-001A**

Origin:

`GPE.UI MFE-001A` human runtime probe, 2026-09-01.

---

## Observed problem

The MFE-001A visual probe exposed two presentation issues that are separate from transaction/layout correctness.

### 1. Missing-glyph behavior is ambiguous

The probe title contained an em dash:

```text
GPE.UI MFE-001A — T1 TRANSACTION
```

The current bitmap-font/text path does not render that character as intended. At runtime it appears as a `?`-like replacement.

That is not a good missing-glyph contract because the user cannot distinguish:

```text
literal '?' character
```

from:

```text
unsupported character fallback
```

GPE should define an explicit, deterministic missing-glyph policy.

Candidate requirement:

> Any unsupported character must render as a recognizable missing-glyph symbol/picto rather than silently degrading to an ambiguous ordinary question mark.

The exact glyph is not selected by this document.

Candidates include a dedicated replacement box/tofu-style glyph or another intentionally designed GPE fallback mark.

---

## 2. Typography quality is insufficient for polished GPE.UI

The current bitmap text is functional and appropriate for some pixel-art/debug use cases, but the MFE probe makes clear that it is visibly coarse for a general-purpose, highly customizable UI subsystem.

Strategic pressure:

```text
GPE.UI should support deliberately pixel-art text when desired,
but should not force every game/tool UI to look heavily pixelated.
```

This is broader than changing one font asset.

A future study should distinguish at least:

- bitmap/pixel font path;
- outline TTF/OTF path already present/evolving in GPE;
- scaling/filtering policy;
- font metrics and text measurement consistency;
- glyph coverage;
- fallback behavior;
- Native/Web behavior;
- dependency/binary-size impact;
- potential future richer rendering techniques only if justified.

Do not jump directly to SDF/MSDF or a large text stack without comparison and measurements.

---

## Required follow-up questions

1. What character repertoire does the built-in bitmap font intentionally support?
2. What is the current behavior for unsupported Unicode scalars?
3. Is there already a dedicated fallback glyph in the bitmap font data that is not being used consistently?
4. If not, what explicit fallback glyph/picto should become the contract?
5. Should unsupported-glyph occurrence be observable in debug/test diagnostics?
6. What is the smallest path to noticeably better non-pixelated UI typography using GPE's TTF/OTF capability?
7. Can bitmap and outline fonts coexist behind one sufficiently small text-measure/paint abstraction without forcing outline-font dependencies on tiny consumers?
8. What differs between Native and Web rendering/metrics?

---

## Acceptance direction for a future slice

A focused future mission should, before broad implementation:

- audit current bitmap-font coverage and fallback behavior;
- reproduce unsupported-character cases with automated tests;
- define one explicit missing-glyph contract;
- verify accents and common UI punctuation (`—`, `–`, arrows/bullets where intentionally supported);
- compare the existing bitmap path with the outline-font path for UI readability;
- keep pixel-perfect bitmap rendering available;
- avoid making a heavy font stack mandatory for consumers that do not use it;
- validate Native and Web code paths separately;
- include a small visual probe for human readability judgment.

---

## Non-goals

This follow-up does not authorize:

```text
full browser-grade text shaping
full CSS typography
arbitrary font-manager globals
large Unicode/i18n redesign
SDF/MSDF by default
rewriting GPE.UI transaction/layout architecture
```

---

## Relationship to MFE-001A

```text
MFE-001A transaction semantics: PASS
text rendering quality: separate follow-up
missing-glyph contract: separate follow-up
```

The typography issue should be tracked independently so it can improve GPE/UI globally without contaminating the falsifiable scope of the transaction experiment.
