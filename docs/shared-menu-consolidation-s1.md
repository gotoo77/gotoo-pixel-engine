# Shared Menu Consolidation S1 — rapport

25 novembre 2026. **IN SCOPE ; NO-GO pour clôturer ou publier S1.**
Travail local partiel, non commité. Un gate de feature échoue aussi sur la base
canonique intacte et trois défauts S1 sont confirmés par revue ; arrêt appliqué sans corriger un exemple annexe ni affaiblir
les critères. Aucun S2, autre dépôt modifié, pin changé, push, merge ou déploiement.

## Base et préflight

- Base et `main` serveur réel : **6ca540f91b1ce8cd1e69f5a93fe71a4ce70dca43**,
  vérifié par `git ls-remote` au début et après les validations. PR Core #113
  confirmée mergée par GitHub. Aucun avancement supplémentaire de main.
- Depuis la référence S0 `8cbceda0c9b0cc55ee5deb7d5e173676ffc1d4f1` : seulement
  #113, sept fichiers, 33 ajouts/18 suppressions (cfg et avertissements WASM,
  exemples de diagnostic/manette, runner CI). UI, ControlMap, PixelGameHost,
  manifeste et lockfile identiques. Le cadrage S1 demeure applicable.
- Worktree géré dédié : `/home/gduval/.codex/worktrees/4497/gotoo-pixel-engine`,
  branche `feat/shared-menu-consolidation-s1`, HEAD encore à la base.
- Checkout original `game/void-canticle` à `39eec6b…` conservé. Son rapport
  `docs/shared-shell-consolidation-audit.md` reste non suivi et identique :
  SHA-256 `5e88c8029826c0f48fc05102675baab751ba2cb5810395e23750685d3b517c67`.
- Worktree S0.1 `/home/gduval/.codex/worktrees/c81d/gotoo-pixel-engine`,
  `fix/wasm-strict-validation` à `babcc5a…`, conservé propre ; son arbre est
  identique à la base canonique. Aucun worktree supprimé ou déplacé.
- Instructions de session et AGENTS applicables, RTK, ICM, Knowledge Vault,
  chartes menu/shell, A9, audit S0 original et préflight Pirates P1.2 lus.
  Aucun AGENTS physique dans Core ou ses sous-répertoires trouvé.
- Gates par défaut exécutés sur la base intacte avant le code : canonique natif
  **393 tests**, Clippy locked strict natif et WASM all-targets, check-web,
  **102 tests JS**, **2 tests Chromium**, build-web : **PASS**.

## Références réellement comparées

| Référence | Constat de sources et décision |
| --- | --- |
| Core `1a461ddf03be63c5d2e4f0f7e555676e9e1c1ae3` | Pile et composition initiales ; opt-in PauseGame et deux tests de suspension/release. Les assertions de ces tests sont adaptées au consommateur unique. L'opt-in wrapper est exclu. |
| Core `cf5d2795edf85404f005e8b89bfb51ed92b9e264` | Source principale : pile, intentions, audio, style, outline, snapshot/hitboxes, reset et tests. |
| Core `05ad42d184dd809e8072bb2d9b3596fef5d48f68` | `menu.rs` et PauseGame identiques à cf5d279. Correction historique du lifetime outline examinée ; adaptation locale sans variante morte ni allow. Sons UI exclus. |
| Arcade main `80a61a2196f9d867a214e24878f74efaa0d4861c` | Modal local existant, FR/EN, quatre gains persistés, pages dynamiques, priorité terminal. Ne consomme pas ui::menu. Aucune migration ni rétrogradation implicite. |
| Arcade PR #14, head `52d9c5ec915a14b320120d8b952bfa8dc513eedc` | Toujours ouverte/draft et non mergée ; pilote Snake consommant le menu, styles/snapshot, géométrie hébergée, erreurs audio et release gate. Ses tests modal/résolution ont été examinés ; pas importés comme code métier Core. |

Comparaison des objets Git et des sources exactes exportées sous `/tmp/gpe-s0-src`.
Métadonnées GitHub et refs serveur relues. Les pins historiques ne sont jamais
présentés comme des bases compatibles avec main ni comme des builds validés.
Leurs changements plateforme, hôte pixel, toucher, diagnostics et dépendances
ne sont pas repris. Le test plateforme obsolète de cf5d279 est exclu.

## Composition locale disponible

- `ui::menu::MenuStack<Id>` et `MenuBack` : racine conservée, état transactionnel
  et focus par page ; `ui_state_mut()` permet au consommateur de composer les
  widgets, labels, valeurs et actions métier existants sans second kernel.
- `PauseSettingsMenu`, `SettingsPage`, `SettingsIntent::{None,Resume,Exit}` :
  composition de référence pause/options/audio. Pas de `Game` implémenté, aucun
  booléen de pause, aucune simulation, sortie de processus ou persistance.
- `AudioSettings { master, music, sfx, ui }` : défauts à 1, pas de 0,05 ; seule
  `apply_audio` applique explicitement les propositions aux backends existants.
  Ordre master/Music/Sfx/Ui, arrêt sur première erreur inchangée, application
  antérieure conservée. Le consommateur reste responsable des valeurs effectives,
  capacités, messages d'erreur et de son propre schéma de préférences.
- `SettingsLabels<'a>` fournit les libellés par transaction, avec défauts anglais
  et clés sémantiques indépendantes de la langue. Pas de système de localisation
  ou de schéma de préférences ajouté.
- Style via UiTheme/UiStyleSheet existants ; outline optionnel via OutlineFont.
  `UiOutlineText` regroupe les paramètres de l'ancien runner sans relaxer Clippy.
- `SettingsPresentationSnapshot` reste justifié par PR #14. Cinq slots au lieu
  de quatre pour conserver le bus Ui. Coordonnées du framebuffer du menu ; le
  consommateur applique sa présentation d'hôte. Les rectangles sont invalidés
  après transition/reset jusqu'au prochain layout. `UiOutput::rect` utilise la
  même géométrie que le kernel et rejette un handle d'une transaction précédente.
- `nav.cancel` prioritaire sur activation/édition simultanée ; transitions après
  la transaction. La revue a trouvé une lacune pour le bouton Back (ci-dessous). Les booléens UiNavInput sont des impulsions : adaptation physique
  et attente de neutralité restent explicitement chez le consommateur.

L'exemple existant `gpe_ui_p6_high_res_presentation` compose ces capacités via
`examples/support/menu_consumer.rs`. Son unique `Phase` possède running/entrée/
menu/sortie. Les contrôles standards existants sont réutilisés. Il n'appelle pas
le monde pendant les menus/gates, attend les sources clavier/manette brutes et
retient la frame de relâchement ; aucune deuxième pause n'est imbriquée.
Exemple Standard clavier/manette, sans adoption tactile ou pointeur implicite.
Exemple visuel sans sons : réglages présentés comme propositions, sans prétendre
modifier un contenu sonore. L'API audio explicite est testée séparément.

Exclusions : opt-in PauseGame historique, nouveau shell/framework/menu principal
figé, règles Pirates/Arcade/terminal, sons UI, stockage, contrôle/capture/remapping,
bindings sérialisés, conflits, profils Touch, migration/pins/inscription, correction
du lock VC, dépendances et changements plateforme/hôte. `PauseGame` legacy,
ControlMap, Input, renderer, Audio et PixelGameHost sont inchangés.

## Tests repris et ajouts

Les sept tests menu historiques par défaut sont repris : pile/racine,
restauration du focus, navigation pause/settings/audio/retour racine, sliders,
hitboxes, reset préservant l'audio et application des bus. Le test outline
historique est aussi conservé. Aucune assertion historique réduite ; seule la
fixture audio reçoit `ui: 1.0` pour conserver son assertion sur le bus Ui.

Les deux tests historiques de suspension/release de 1a461dd sont adaptés dans
`src/ui/menu_consumer_tests.rs` à la composition du véritable exemple, plutôt
qu'à l'opt-in wrapper exclu. Trois tests supplémentaires y couvrent entrée avec
confirm maintenu, absence de seconde activation, sources clavier/deux pads,
déconnexion, frame neutre, navigation pad et intention Exit.

Douze tests publics couvrent composition de pages métier, pile, intentions,
application explicite des quatre gains, focus/retour et invalidation de géométrie,
bornes/opposés des sliders, priorité cancel, pointeur/touch et activation simultanée,
habillage/pixels/libellés/redimensionnement, handles anciens, erreurs exactes à
chaque setter et application partielle, valeurs invalides et reset des quatre gains.
Un treizième sous outline vérifie typographie, normalisation de taille non finie
et interaction/hitboxes communes.

Défaut : **417 = 393 + 7 menus historiques + 5 consommateur + 12 publics**.
Outline : **434 tests** (inclut aussi les tests préexistants propres à la feature).
Zéro ignoré/échec dans ces suites. Les corps des tests existants de
`ui/experimental.rs` et `ui/mod.rs` sont inchangés ; autres sources de tests,
JS, CI et scripts de validation inchangés. Les 102 tests JS restent actifs.
Une première compilation des contrats publics échoue bien sur l'absence de
`ui::menu` avant reprise ; le consommateur est également testé avant création.
Des erreurs intermédiaires d'adaptation de fixture/appel de peinture ont été
corrigées ; les résultats ci-dessous sont les dernières exécutions.

## Validation et blocage

Linux/WSL, toolchain du dépôt Rust 1.97.1, défauts vides et feature outline-fonts.
Les commandes de validation sont exécutées avec RTK. Préparation JS locale :
`npm install --no-audit --no-fund --package-lock=false` **PASS**, sans lock ajouté.

| Commande / contrôle | Résultat local S1 |
| --- | --- |
| `python3 scripts/dev.py check` | **PASS** : A9, fmt, 417 tests, Clippy natif strict, diff-check. |
| `cargo clippy --locked --all-targets -- -D warnings` | **PASS — Clippy natif**. |
| `cargo clippy --locked --all-targets --target wasm32-unknown-unknown -- -D warnings` | **PASS — Clippy WASM** ; compilation/analyse des targets, pas exécution des tests Rust dans le navigateur. |
| `python3 scripts/dev.py check-web` | **PASS**, compilation WASM web_demo. |
| `python3 scripts/dev.py check-web-unit` | **PASS**, 102 tests / 8 fichiers. |
| `python3 scripts/dev.py build-web` | **PASS**, cette commande compile web_demo ; pas un déploiement ou packaging release. |
| `python3 scripts/dev.py check-web-browser` | **PASS**, deux tests Chromium réels (startup winit/WASM et texte DOM↔Rust). Ils ne testent pas le nouveau menu. |
| `cargo test --locked --lib --bins --examples --tests --features outline-fonts` | **PASS**, 434 tests. |
| `cargo clippy --locked --all-targets --features outline-fonts -- -D warnings` | **FAIL**, exit 101 ; échec préexistant reproduit à l'identique sur base canonique intacte. |
| `cargo clippy --locked --all-targets --features outline-fonts --target wasm32-unknown-unknown -- -D warnings` | **NOT RUN**, arrêt au gate natif de feature. |
| `cargo build --locked --example gpe_ui_p6_high_res_presentation` | **NOT RUN** en commande dédiée ; exemple compilé/testé via le gate natif. |
| `cargo build --locked --target wasm32-unknown-unknown --example gpe_ui_p6_high_res_presentation` | **NOT RUN** ; code analysé par Clippy WASM all-targets, sans build dédié revendiqué. |
| `cargo test --locked --doc` | **NOT RUN**, arrêt avant cette étape ; recette ajoutée non revendiquée comme doctest exécuté. |
| `cargo fmt --all -- --check` | **PASS**, vérification explicite finale. |
| `git diff --check` | **PASS**, vérification explicite finale. |
| Cargo.toml/Cargo.lock, CI, scripts, anciens tests, sources hors scope | **PASS**, comparés à la base ; aucune modification. |
| Exécution réelle du nouveau menu natif/Web ; clavier/manette physiques ; Windows | **NOT RUN**. Tests d'entrées synthétiques uniquement ; aucune promesse matérielle/runtime. |
| Intégrations Pirates/Arcade, CI distante, publication/déploiement | **NOT RUN**, hors scope. |

**Blocage précis :** `examples/gpe_ui_p4_font_gallery_web.rs:38`, fonction
native `main` inutilisée dans l'exemple déclaré `crate-type = ["cdylib"]` et
activé par outline-fonts. Diagnostic `dead_code`, interdit par `-D warnings`.
Ce fichier, le manifeste et le lock sont identiques à la base. La commande
exacte a été reproduite sur un `git archive` neuf de
`6ca540f91b1ce8cd1e69f5a93fe71a4ce70dca43` sous
`/tmp/gpe-s1-outline-baseline` : même ligne, même diagnostic, exit 101.
Logs : `/tmp/gpe-s1-outline-native-clippy.log` et
`/tmp/gpe-s1-baseline-outline-clippy.log`. Les autres logs S1 sont sous
`/tmp/gpe-s1-*.log` ; ils restent des artefacts temporaires.

Aucun allow/expect, désactivation de target/test, retrait de lint, modification
CI, abandon d'outline pour masquer le gate ni correction annexe effectué.
Le hard stop de la mission interdit de qualifier cette consolidation de validée.

## Revue indépendante

Revue indépendante en lecture seule, réalisée par un second agent sur le diff
et les nouveaux fichiers, avec sondes autonomes utilisant l'API publique.
**FAIL / corrections requises**, en plus du gate préexistant :

1. **Important — capture parent conservée après push** (`src/ui/menu.rs:72`,
   transition à `:453`). Sur Pause, maintenir le pointeur sur Settings et confirmer
   au clavier ; revenir avec cancel tout en gardant le pointeur maintenu ;
   relâcher sur Settings. Une seule pression pointeur active à nouveau la page :
   résultat `Settings`, attendu `Pause`. La capture du parent reste dans son
   UiStateStore suspendu. Corriger à la reprise en conservant le focus tout en
   neutralisant les captures transitoires lors d'un changement de page ; tester
   pointeur et toucher. Le test actuel de focus ne couvre pas ce scénario mixte.
2. **Important — Back peut aussi éditer l'audio** (`src/ui/menu.rs:431`).
   Presser Back dans Audio, puis relâcher dessus avec `nav.up + nav.left`.
   Le menu revient à Settings mais le gain Ui passe de `1.0` à `0.95`, les
   propositions étant commitées avant la détection du bouton Back à `:443`.
   Corriger en donnant priorité à toute forme de retour avant d'accepter les
   changements ; ajouter le scénario mixed-input. `nav.cancel` est bien couvert,
   l'activation Back concomitante ne l'est pas.
3. **Important — repli/troncature des libellés outline**
   (`src/ui/experimental.rs:1725`). Le texte mesuré dans le contrôle est redessiné
   avec sa seule largeur d'encre, provoquant un second wrapping. Avec Exo2 à
   14px/1×, QUIT/AUDIO passent de 17px de hauteur dans 600px disponibles à 34px
   avec les largeurs 32/42px fournies au dessin, dépassant la ligne de 24px.
   À 20px, RESUME/SETTINGS sont également touchés. Préserver une largeur de layout
   suffisante en centrant l'origine et ajouter une régression de rendu texte ;
   le test outline actuel ne prouve que le contrat d'interaction.

Sondes indépendantes `/tmp/gpe-s1-independent-mixed-input` et
`/tmp/gpe-s1-independent-outline` exécutées aussi par l'agent principal : mêmes
observations. Ce sont des **reproductions de défauts**, pas des tests acceptés
comme PASS. Le gate outline préexistant est confirmé séparément sur archive
canonique. Aucun correctif produit ou test en dépôt ajouté après le hard stop.
Les trois défauts sont des conditions de reprise obligatoires, pas des détails
facultatifs ou des limitations dissimulées par les suites actuellement vertes.

## Fichiers et suite proposée

Modifiés : `src/ui/experimental.rs`, `src/ui/mod.rs`,
`examples/gpe_ui_p6_high_res_presentation.rs`.

Ajoutés : `src/ui/menu.rs`, `src/ui/menu_consumer_tests.rs`,
`tests/shared_menu.rs`, `examples/support/menu_consumer.rs`, ce rapport et
`docs/superpowers/plans/2026-11-25-shared-menu-consolidation-s1.md`.
Tous restent non committés dans le worktree dédié. S0 et S0.1 préservés.

**Surface pour S2 :** composition locale de pages via MenuStack et widgets
existants, style/outline/snapshot et intentions. Aucun contrat de remapping,
capture, codec, conflit, defaults de bindings ni modification de ControlMap.
Cette surface est une proposition locale non figée tant que S1 n'est pas validé.

**Verdict NO-GO** : réparer séparément le gate outline natif préexistant, puis
réauditer/rebaser S1, corriger les trois défauts de revue avec régressions et
terminer gates de feature, builds dédiés et doctest.
Les essais runtime/matériel et les migrations restent explicitement distincts.
S1 seul ne débloque pas Pirates P1.2 ; S2 et les intégrations autorisées restent
nécessaires. Aucune correction annexe n'a été entreprise.

Proposition de commit/PR S1, **après résolution du blocage et validation complète** :
`feat(ui): consolidate shared menu stack and pause settings`.
Description : reprendre les menus partagés sur main, avec état modal au
consommateur, quatre gains audio explicites, habillage et géométrie communes ;
conserver les capacités main et tests historiques. Aucun commit ou PR créé.
