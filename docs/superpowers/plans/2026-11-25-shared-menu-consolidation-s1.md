# Shared Menu Consolidation S1 Implementation Plan

> Execute inline with superpowers:executing-plans; one independent final review. No commit, push or merge.

**Goal:** Consolidate existing shared menus on canonical Core without S2 or consumer migration.

**Architecture:** Selective transplantation from cf5d279/05ad42d onto 6ca540f91b1ce8cd1e69f5a93fe71a4ce70dca43, using existing transactional UI, navigation, styles, fonts and audio. The consumer owns modal transitions, simulation, release gates and preference schemas.

**Tech Stack:** Existing Rust GPE, optional outline-fonts and canonical Python/Web checks.

**Spec:** User mission GPE S1; original docs/shared-shell-consolidation-audit.md and Pirates docs/p12-preflight.md read without modification.

## Constraints and design

Preserve original checkout/S0 and S0.1. No S2, ControlMap changes, manager/storage, dependencies, pins/migrations or unrelated platform/host/touch/diagnostics changes. Preserve all baseline tests and modern main capabilities, including legacy PauseGame.

Selective transplantation is required. Global merge loses main changes; a new shell crate/framework has no demonstrated need. MenuStack exposes each page state to the existing builder: the extension seam for consumer pages/actions, without another focus authority. PauseSettingsMenu returns None/Resume/Exit; it has no Game implementation or pause flag. Keep the snapshot used by Arcade PR14 and extend to five controls for Core's Ui bus. Styles and borrowed labels permit consumer presentation; localization policy stays outside Core. Outline rendering uses existing OutlineFont. Audio application stays explicit and fallible with documented partial application. Cancel wins simultaneous editing/activation.

## Review focus

- Raw held keyboard and multiple-pad sources across entry/resume; aggregated edges alone are insufficient.
- Simultaneous cancel/confirm/pointer: one intent, no value edit on back.
- Parent focus restoration and geometry invalidation on transition/resize.
- Errors at each audio setter, partial state, no implicit application.
- Preserve bitmap/outline/host/input capabilities; exclude obsolete historical platform tests.

## Tasks

### 1. Components and kernel prerequisites

Files: src/ui/menu.rs, src/ui/mod.rs, src/ui/experimental.rs, tests/shared_menu.rs.
Interfaces: MenuStack/MenuBack, SettingsPage/SettingsIntent, AudioSettings(master/music/sfx/ui), PauseSettingsMenu, UiOutput::rect, outline transaction.

- [ ] Add public API tests; observe absent API failure.
- [ ] Restore historical menu tests without reducing assertions; adapt canonical fixtures.
- [ ] Import only menu geometry/outline prerequisites, excluding touch/platform/host/audio sounds.
- [ ] Test focus, transitions, styles, hitboxes, cancel precedence, four volumes, backend errors.
- [ ] Run default/outline tests and strict checks.

### 2. Consumer ownership demonstration

Files: existing examples/gpe_ui_p6_high_res_presentation.rs and tests.
Consume MenuStack/PauseSettingsMenu and existing navigation/input APIs.

- [ ] Adapt historical 1a461dd suspension/release assertions to one consumer modal state.
- [ ] Prove nested settings, suspension, no double activation, keyboard/pad neutralization, Exit.
- [ ] Compile native/WASM; distinguish runtime evidence.

### 3. Validation, independent review, report

- [ ] Canonical native, strict locked Clippy native/WASM all-targets, web checks/unit/browser/build, outline feature checks.
- [ ] Check all baseline tests retained, fmt, lock, diff and checkout preservation.
- [ ] Independent scope/API/modal/input/main-regression review; regression tests for material findings.
- [ ] Report sources, choices, files, PASS/FAIL/NOT RUN, S2 surface, verdict and commit/PR proposal without publishing.

## Ledger / arrêt

Composants, reprise des tests, composition consommateur et tests ciblés : réalisés.
Gates défauts PASS417Rust/102JS/2Chromium + Clippy strict natif/WASM.
Tests outline PASS434 ; Clippy outline natif FAIL préexistant sur6ca540f (galerie Web main inutilisé).
Hard stop : aucune nouvelle modification de code ; validations restantes NOT RUN.
Revue indépendante demandée ; rapport docs/shared-menu-consolidation-s1.md.
Pas de commit/push/merge. Scope encore incomplet à cause du blocage de validation.

Revue indépendante : trois Important reproduits (captures parent maintenues,
édition audio concomitante à Back, wrapping/troncature outline). Non corrigés
après le hard stop ; régressions et corrections obligatoires à la reprise.
Verdict NO-GO ; aucune surface S2 figée ou adoption consommateur autorisée.
