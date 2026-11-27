# GPE #117 — correction v2 après playtest

Contrat unique de cette correction : [v2](https://github.com/gotoo77/gotoo-pixel-engine/issues/117#issuecomment-7229097296).
Base exacte, initialement propre : `cd515837939126f5bf430436338e30beab7f0a6f`.
Branche : `feat/ui-resolved-targets-117`. Rust : `1.97.1`.
Ce rapport décrit le commit qui le contient, successeur direct de cette base.
La qualification v1 reste une référence historique, pas une validation runtime v2.

## Modifications distinguées

- **A** : `kernel.rs` ajoute une préférence de cône, sélectionnée seulement par
  `resolved`. Demi-plan demandé, puis priorité à `primary >= secondary`, puis
  distances primaire/secondaire et ordre stable. Sans candidat dans le cône,
  ancien fallback déterministe. Aucun second noyau. Les anciens modes Spatial
  et Linear conservent leur comportement.
- **B** : `Input` expose le focus réel et une génération durable de reset,
  préservés par Clone/advance_frame et inclus dans PartialEq. Les fenêtres
  principale et outil initialisent le focus réel et transmettent gain/perte.
  Le reset conserve son effacement historique des périphériques de fenêtre,
  sans faux release/end, et préserve les gamepads. La synchronisation publique
  `ResolvedState::synchronize_input` appelle suspend avant les événements,
  conserve le focus logique et impose neutralité observable puis sémantique.
  Le probe efface le flash et bloque ses transitions pendant cette période.
- **C** : Enter rejoint localement Space/South pour CONFIRM dans le probe.

Aucun autre adaptateur, mapping, renderer, host, manifeste Cargo, dépendance,
Pirates ou GPE-Agent modifié. Aucun merge, rebase, changement de pin ou S2.

## Preuves discriminantes

Avant correction A, les nouveaux tests échouent sur les rectangles exacts du
probe : Haut choisit Droite ; le cas frontière préfère aussi le hors-cône.
Après correction, les quatre directions passent, Down activé/désactivé,
sans activation pendant navigation. Frontière incluse, hors-cône seul,
disabled, reorder, ties et distance secondaire sont couverts. Un test du
passage partagé compare explicitement Spatial, Linear et la nouvelle politique.
Tous les anciens tests sont conservés, sans changement de leurs assertions.
La politique n'est pas réciproque : Droite puis Gauche choisit Haut, 2 pixels
plus proche sur l'axe primaire avec les centres entiers historiques du noyau.
Les retours opposés applicables reviennent au centre. Ce cône est une correction
minimale du défaut observé, pas une preuve d'ergonomie universelle.

Avant correction B, le test utilisant le vrai `Input::reset_window_devices`
laisse la capture active : assertion `!out.active(target.id)` en échec.
Le test corrigé passe par la synchronisation publique. Les tests publics
exercent aussi perte/reprise avant update, snapshot cloné, captures souris/touch,
confirm concurrent, inactivité non focalisée, fins héritées avant/après neutralité,
nouveaux clic/touch et confirm. Les tests Input vérifient la durabilité,
l'absence de faux releases, les données préservées et les held réellement
observables. Sortie du pointeur et frame tactile vide ne sont pas une perte OS.

Mutation temporaire supplémentaire : suppression de l'incrément durable du
reset. Le test public échoue sur `!state.synchronize_input(&input.clone())`
pour perte+reprise entre updates. Source restaurée dans un bloc finally avant
les gates finaux. Log local : `/tmp/gpe117-v2/lifecycle-mutant.log`.
Une revue indépendante en lecture seule ne relève aucun défaut critique ou
important ; elle ne qualifie ni le runtime ni l'exécution des gates.

## Commandes réellement exécutées

Toutes les commandes utilisent le proxy `rtk`. Les logs de la matrice sont
locaux dans `/tmp/gpe117-v2/matrix-*.log`.

- **PASS** `cargo test --locked --test ui_resolved` — 22 tests publics.
- **PASS** `cargo test --locked --test ui_resolved --all-features` — 22 tests.
- **PASS** `cargo test --locked --lib input::tests` — 22 tests ciblés.
- **PASS** `cargo test --locked --lib cone_policy_does_not_change_historical`.
- **PASS** `cargo fmt --all -- --check` ; `git diff --check`.
- **PASS** `python3 scripts/dev.py check` — 457 tests dans 27 suites,
  séparation, rustfmt, Clippy strict `--all-targets -- -D warnings`, diff.
- **PASS** `python3 scripts/dev.py check-web` — build Web canonique.
- **PASS** `cargo check --locked --lib --test ui_resolved --example gpe_ui_resolved --all-features`.
- **PASS** `cargo clippy --locked --lib --test ui_resolved --example gpe_ui_resolved --features outline-fonts -- -D warnings`.
- **PASS** même Clippy ciblé avec `--all-features`.
- **PASS** mêmes Clippy ciblés avec `--target wasm32-unknown-unknown`,
  respectivement features par défaut, `outline-fonts`, et `--all-features`.
- **PASS** `cargo build --locked --example gpe_ui_resolved --features outline-fonts`,
  puis même build avec `--target wasm32-unknown-unknown`.
- **PASS** `wasm-bindgen --target web --out-dir /tmp/gpe117-v2/web target/wasm32-unknown-unknown/debug/examples/gpe_ui_resolved.wasm`.
- **FAIL historique, NOT RUN dans cette session** : Clippy global
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  conflit d'allocateurs préexistant [#112](https://github.com/gotoo77/gotoo-pixel-engine/issues/112),
  déjà reproduit sur la base et consigné dans le rapport v1. Aucun affaiblissement
  ni nouvelle investigation de #112. Les cibles de cette correction passent
  Clippy strict avec toutes les features.

## PR ouvertes : inspection ciblée

Diffs ouverts inspectés avant modification Input/platform :
[#107](https://github.com/gotoo77/gotoo-pixel-engine/pull/107),
[#108](https://github.com/gotoo77/gotoo-pixel-engine/pull/108),
[#110](https://github.com/gotoo77/gotoo-pixel-engine/pull/110).
Leurs changements de coordonnées et gamepad virtuel sont distincts des hunks
lifecycle de cette correction. Le mapping de #107 clone Input et préserve ainsi
ces métadonnées ; #110 transforme les positions sans toucher au lifecycle.
Proximité textuelle dans Input/platform à surveiller à l'intégration, sans
chevauchement non trivial constaté. Aucune PR intégrée ni rebasée.
Les autres PR ouvertes inspectées, #111 (audio), #106 (fullscreen Web) et
#101 (CI), ne modifient aucun fichier de ce delta. Cette inspection n'est pas
une preuve de merge futur : aucun merge de qualification n'a été exécuté.

## Limites et retest humain requis — runtime HOLD

**NOT RUN** : playtest OS réel natif, gamepad/touch physiques et Web interactif.
Aucun diagnostic headless relancé. Aucune nouvelle preuve de FPS ou de rendu.
Le snapshot de perte de focus efface les held clavier/souris/touch : leur
relâchement physique hors fenêtre n'est donc pas prouvable. Le système garantit
l'annulation des captures et l'absence d'activation par release/end hérité ; un
nouveau press/start est nécessaire. Les gamepads restent observables et bloquent
la reprise tant que boutons/axes ne sont pas neutres ; le drift d'un axe peut
conservativement retarder la reprise. Une frame tactile vide ne prouve pas la
fin d'un contact physique. Ces limites sont documentées dans l'API.

Depuis le SHA livré :

```sh
rtk cargo run --locked --example gpe_ui_resolved --features outline-fonts
```

1. Depuis Center, vérifier Haut→Up, Droite→Right, Gauche→Left ; activer Down
   avec D puis vérifier Bas→Down. La navigation seule ne produit aucun flash.
   Depuis Up/Left/Down, vérifier le retour opposé vers Center ; le cas
   Right→Left sélectionne Up selon le classement ci-dessus.
2. Presser un bouton à la souris, Alt+Tab, relâcher ailleurs, revenir : aucune
   capture verte ni flash/action. Attendre la neutralité, puis nouveau clic :
   une seule confirmation. Refaire avec loss+gain rapide, Enter/Space/South
   concurrent, D et R pendant la reprise ; aucune transition héritée.
3. Refaire avec contact tactile interrompu ; vérifier que sa fin héritée ne
   confirme rien et qu'un nouveau contact fonctionne. Avec gamepad maintenu
   pendant Alt+Tab, vérifier l'attente de neutralité puis un geste frais.
4. Répéter navigation et blur/focus dans le vrai navigateur avec le même probe
   WASM, à partir du harnais manuel existant. Ne pas conclure depuis un screenshot
   headless. Les commandes de reconstruction sont les build/bindgen ci-dessus.

Le retest humain natif **et** Web reste nécessaire. Aucun GO S2.
