# GPE #117 v3 — classement spatial uniquement

Contrat : [v3](https://github.com/gotoo77/gotoo-pixel-engine/issues/117#issuecomment-7229395736).
Base exacte, worktree initial propre : `7e123f23424e7fe102d80861b107b4d467dd5616`.
Branche conservée : `feat/ui-resolved-targets-117`.
Le candidat est le commit contenant ce rapport, successeur direct de cette base.

## Delta et preuves

Le passage partagé SpatialCone, choisi exclusivement par resolved, privilégie
les candidats dans le cône dont les projections perpendiculaires se recouvrent
strictement : Y pour gauche/droite, X pour haut/bas. Les autres candidats dans
le cône suivent, puis les candidats hors cône selon le fallback v2 inchangé.
Chaque groupe conserve distances primaire/secondaire et ordre stable.
Intervalles semi-ouverts, bords en i64, aucun float ni historique ajouté.

Avant toute modification de production, le test du probe exact a été changé
pour exiger le retour à Center dans les quatre directions activées : échec
constaté sur Right→Left, résultat Up au lieu de Center. Les tests de préférence
alignée et de recouvrement ont aussi échoué avant correction. Les assertions
de non-activation et Down désactivé sont conservées.

Le test de ties distingue explicitement la nouvelle préférence alignée avec
tous les candidats, puis conserve les attentes d'ordre/primaire entre diagonales
avec la cible alignée désactivée. Les nouveaux cas couvrent recouvrement d'un
pixel et contact de bord sur les deux axes, aligné désactivé, alignement hors
cône non prioritaire, fallback uniquement hors cône et bords dépassant i32.
Le test partagé historique vérifie aussi Right→Left : Spatial conserve Up,
Linear conserve Right, seul resolved revient à Center.

Input, platform, resolved et ses captures/lifecycle, géométrie du probe et
autres adaptateurs sont identiques à la base. Aucun changement de dépendance,
renderer, host, Pirates, pin ou S2. Aucun merge ni investigation Web.

## Validation automatisée

Commandes via `rtk` ; logs locaux dans `/tmp/gpe117-v3/gate-*.log` et
`/tmp/gpe117-v3/final-*.log`. Une revue indépendante en lecture seule ne relève
aucun défaut critique ou important ; elle ne qualifie pas le runtime.

- **PASS** `cargo test --locked --test ui_resolved` : 24 tests publics.
- **PASS** même commande avec `--all-features` : 24 tests.
- **PASS** `cargo test --locked --lib cone_policy_does_not_change_historical`.
- **PASS** `python3 scripts/dev.py check` : 459 tests dans 27 suites, séparation,
  rustfmt, Clippy strict `--all-targets -- -D warnings` et diff.
- **PASS** `python3 scripts/dev.py check-web` : compilation Web canonique.
- **PASS** `cargo check --locked --lib --test ui_resolved --example gpe_ui_resolved --all-features`.
- **PASS** `cargo clippy --locked --lib --test ui_resolved --example gpe_ui_resolved --features outline-fonts -- -D warnings`.
- **PASS** même Clippy natif avec `--all-features`.
- **PASS** Clippy ciblé avec `--target wasm32-unknown-unknown` pour features
  par défaut, `outline-fonts`, puis `--all-features`, toujours `-D warnings`.
- **PASS** `cargo fmt --all -- --check` et `git diff --check`.
- **FAIL historique / NOT RUN v3** : Clippy global `--all-targets --all-features`,
  conflit d'allocateurs préexistant #112 consigné dans les rapports précédents.
  Aucun assouplissement de lint ou gate ; les cibles de v3 passent strictement.

## Qualification humaine et retest

Le propriétaire a validé sur **v2** : quatre directions depuis Center,
annulation de capture Alt+Tab, nouveau clic après retour et Enter. Le retour
Right→Left de v2 a été rejeté. Ces résultats ne constituent pas un playtest v3.

**NOT RUN — runtime v3**, navigateur réel et pad/touch physiques. Aucun nouveau
playtest n'est revendiqué. Pas de promesse de réciprocité universelle : le test
discriminant porte sur la géométrie exacte du probe. Retest natif demandé :

```sh
rtk cargo run --locked --example gpe_ui_resolved --features outline-fonts
```

Activer Down avec D. Depuis Center, vérifier successivement Right→Left,
Left→Right, Up→Down et Down→Up : retour à Center sans flash d'activation.
Refaire avec Down désactivé et vérifier son exclusion. Vérifier brièvement
pression → Alt+Tab → relâchement ailleurs → retour, puis nouveau clic : capture
annulée et nouveau geste fonctionnel. Ensuite, retester dans le navigateur réel
avec le probe existant. Runtime v3 en attente de ce retest ; aucun GO S2.
