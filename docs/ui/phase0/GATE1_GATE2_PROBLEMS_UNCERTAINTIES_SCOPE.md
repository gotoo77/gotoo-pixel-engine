# GPE.UI Phase 0 — Gates 1–2 : problèmes, incertitudes et réduction du périmètre

## Statut et limites de ce document

Ce document exécute uniquement :

1. Gate 1 — observed problems ;
2. Gate 2 — top uncertainties / uncertainty budget ;
3. research scope reduction.

Il ne contient ni prior art, ni comparaison Architecture 0/A/B/C, ni DAG, ni modèle de transaction ou d'événements, ni étude markup/SVG, ni MFE, ni implémentation.

## Baseline et checkpoint approuvé

| Champ | Valeur |
|---|---|
| Repository principal | `https://github.com/gotoo77/gotoo-pixel-engine` |
| Branche | `research/gpe-ui-phase0` |
| HEAD de départ de cette exécution | `27af0fd096878ea84da5f41716c7dde7a031a6b7` |
| Gate 0 | **APPROVED AND FROZEN** |
| Document Gate 0 | `docs/ui/phase0/GATE0_EXISTING_SYSTEM_AUDIT.md` |
| Baseline code auditée par Gate 0 | `6ff4f8baddae269baa6a7d182f0ba0c9d985f886` |
| Worktree au départ | propre, aligné sur `origin/research/gpe-ui-phase0` |
| Date | `2026-08-31` (`Europe/Paris`) |

Les conclusions de Gate 0 sont réutilisées sans refaire l'audit : identité ordinale avec reset explicite, divergence entre l'input physique du toolkit `Ui` et le chemin `ActionId`/`ControlMap`/`VirtualPad`, et layout `Ui` limité à une colonne racine.

## Pression consumer autonome : GPE Arcade

### Baseline et méthode

| Champ | Valeur |
|---|---|
| Repository | `https://github.com/gotoo77/gpe_arcade` |
| Ref inspectée | `refs/heads/main` |
| Commit exact | `dfcb1c2dee8575e80b37ce99fba2dd38864946ff` |
| Disponibilité | `REMOTE`, inspection Git read-only via copie bare temporaire |
| Mutation du repository consumer | aucune : pas de branche, commit, push ou PR |

L'inspection a été limitée aux sections qui établissent la pression Card UI A3 et aux modules actuels de catalogue/launcher :

- `ARCHITECTURE.md`, sections `Architecture Input` (ligne 219) et `Layout Responsive Des Cartes` (ligne 279) ;
- `DECISIONS.md`, décisions `0006 Modele D'Input Des Cartes` (ligne 73) et `0007 Pagination Avant Scrolling` (ligne 87) ;
- `MIGRATION_PLAN.md`, section `A3 Card UI, Artwork, Mouse Et Touch` (ligne 112) ;
- `examples/arcade/catalog.rs`, `ArcadeGameId` ligne 8, `ResolvedArcadeCatalog` ligne 113 et tests associés ;
- `examples/arcade/game.rs`, `ArcadeLayout` ligne 66, `ArcadeApp` ligne 143, `update_catalog` ligne 177, `launch` ligne 242, `render_catalog` ligne 260 et `catalog_controls` ligne 314 ;
- `examples/arcade/registry.rs`, `BUILTIN_REGISTRY` et `builtin_registry`.

### Faits consumer retenus

1. Le catalogue actuel est encore une liste texte verticale. `MIGRATION_PLAN.md:116` demande explicitement de la remplacer en A3 par une grille de cartes paginée avec interaction complète.
2. Les critères A3 observables sont : sélection clavier/gamepad, hover/click souris, lancement tactile et pagination sur surfaces small/medium/wide (`MIGRATION_PLAN.md:131-135`).
3. Le consumer a déjà décidé qu'une page de layout produit des `Rect` fixes à partir de la taille framebuffer, du ratio/minimum de carte, des gutters et du nombre d'entrées (`ARCHITECTURE.md:279-291`). Il préfère une grille déterministe paginée à un layout CSS ou au scrolling.
4. Le rectangle complet de la carte est la hit target et le hit testing est explicitement possédé par `gpe_arcade::layout`, pas par GPE (`ARCHITECTURE.md:240`).
5. Le document consumer affirme que les APIs GPE actuelles couvrent déjà `ControlMap`, bindings menu, souris, touch, `VirtualPad` et mapping `Viewport` (`ARCHITECTURE.md:219-240`). La décision `0006` retient un modèle unique de sélection multi-input sans demander un framework UI par contrôle.
6. Le catalogue résolu possède déjà des identités stables `ArcadeGameId`, un ordre déterministe et un test `default_ids_are_stable` (`examples/arcade/game.rs:423`). Le nombre d'entrées est fixé lors de `ArcadeApp::new`; le test `menu_count_follows_resolved_catalog` (`game.rs:519`) couvre les catalogues filtrés.
7. L'A3 n'est pas implémentée à ce commit. `ArcadeLayout` contient encore deux géométries explicites Standard/Touch, `MenuState` porte une sélection par index et les boutons tactiles restent les zones UP/PLAY/DOWN.
8. La condition STOP du consumer est étroite : si A3 révèle une primitive générique réellement manquante dans GPE pour le rendu/input, arrêter et proposer séparément le plus petit changement GPE (`MIGRATION_PLAN.md:137-143`).

### Effet sur Gates 1–2

Cette preuve **ne transforme pas** les limites Gate 0 en problème structurel GPE. Elle réduit au contraire le périmètre : Arcade possède le layout, le hit testing, l'artwork, la pagination et la politique de sélection. L'incertitude restante vient du fait que ce contrat accepté n'est pas encore concrétisé contre toutes les traces d'entrée A3 ; elle ne constitue pas une preuve qu'une primitive GPE manque.

## Gate 1 — observed problems

### Réponse obligatoire

> Quel est aujourd'hui le plus petit problème UI structurel réel de GPE que nous devons résoudre ?

**Aucun problème structurel GPE n'est démontré à ce stade.**

Le problème produit observable est local à Arcade : son rendu courant est une liste texte, alors que l'A3 acceptée demande une grille de cartes paginée. Le consumer a déjà attribué cette responsabilité à son propre layout et indique que les primitives GPE existantes couvrent a priori l'input et le rendu nécessaires.

### Problème réel retenu, mais local au consumer

| Champ | Évaluation |
|---|---|
| **Problem** | Le catalogue autonome ne satisfait pas encore son objectif A3 : liste texte actuelle au lieu d'une grille de cartes paginée multi-input. |
| **Evidence** | `gpe_arcade@dfcb1c2`, `MIGRATION_PLAN.md:112-143`; `examples/arcade/game.rs:66-137` et `260-296`. |
| **Consumer(s)** | GPE Arcade autonome uniquement. |
| **Current workaround** | Deux layouts explicites Standard/Touch, une liste verticale via `draw_menu_item`, `MenuState` et trois zones tactiles UP/PLAY/DOWN. |
| **Impact** | L'objectif Card UI A3 et ses critères souris/touch/pagination ne sont pas encore livrés. Aucun impact moteur plus large n'est observé. |
| **Structural or local?** | **LOCAL / PRODUCT-SPECIFIC.** Le repository consumer assigne layout, hit test, pagination et artwork à Arcade. |
| **Could a targeted change solve it?** | **Oui, selon les preuves disponibles.** Le plan A3 décrit un changement ciblé dans `gpe_arcade`; sa condition STOP exige une preuve concrète avant toute demande GPE. Cette possibilité n'est pas encore validée par l'implémentation A3. |

### Candidats Gate 0 non promus en problèmes structurels

#### C1 — Réaffectation de l'identité ordinale

| Champ | Évaluation |
|---|---|
| **Problem candidate** | Focus/capture/repeat de `UiState` peuvent se rattacher à un autre widget après changement structurel sans `reset_interaction()`. |
| **Evidence** | Gate 0 approuvé : `src/ui/ordinal_identity_tests.rs`; tool probe qui applique le reset lors du changement d'onglet. Arcade autonome : `ArcadeGameId` stable et catalogue résolu avant construction d'`ArcadeApp`. |
| **Consumer(s)** | Toolkit `Ui` du tool probe ; pas le Card UI Arcade planifié, qui ne choisit pas `Ui`. |
| **Current workaround** | `UiState::reset_interaction()` pour les pages du toolkit ; Arcade garde actuellement une sélection indexée dans un catalogue stable. |
| **Impact** | Réel pour une structure `Ui` dynamique qui oublie le reset ; aucun échec de consumer actuel observé. |
| **Structural or local?** | **NON PROMU.** Contrainte du toolkit actuel, pas problème structurel GPE démontré. |
| **Could a targeted change solve it?** | Potentiellement oui, mais aucune modification n'est justifiée sans consumer qui échoue au contrat actuel. |

La présence d'`ArcadeGameId` rend possible une restauration locale par identité si A3 en a besoin, mais ne prouve pas qu'un `UiId` générique est nécessaire.

#### C2 — Input physique direct dans `Ui`

| Champ | Évaluation |
|---|---|
| **Problem candidate** | Le toolkit générique lit clavier/souris directement, alors que les UI jeu utilisent des actions sémantiques et le virtual pad. |
| **Evidence** | Gate 0 approuvé. Arcade autonome `ARCHITECTURE.md:219-240`, `DECISIONS.md:73-85`, et `examples/arcade/game.rs:177-224,314-316`. |
| **Consumer(s)** | Tool probe pour `Ui`; Arcade/pause pour `ControlMap`/`VirtualPad`. |
| **Current workaround** | Chaque famille utilise le chemin adapté : input physique pour l'outil desktop, actions/virtual pad pour les UI jeu. |
| **Impact** | Deux styles d'intégration existent ; aucune duplication fautive ni impossibilité produit n'est démontrée. |
| **Structural or local?** | **NON PROMU.** Écart de couverture/API, sans problème architectural consumer-backed. |
| **Could a targeted change solve it?** | Oui si un consumer le réclame : adaptation vers des intentions UI ou paramètres d'action ciblés. Le besoin n'est pas encore établi. |

Arcade affirme que les primitives input actuelles suffisent et conserve sa politique de sélection. Cette nouvelle preuve réduit, plutôt qu'elle n'augmente, la justification d'une abstraction générique immédiate.

#### C3 — Colonne racine unique du toolkit

| Champ | Évaluation |
|---|---|
| **Problem candidate** | `Ui` n'expose ni grille, ni row/stack, ni sous-layout custom. |
| **Evidence** | Gate 0 approuvé. Arcade autonome `ARCHITECTURE.md:279-299` et `DECISIONS.md:87-101`. |
| **Consumer(s)** | Tool probe, satisfait par une colonne ; Arcade, qui possède explicitement son layout de cartes. |
| **Current workaround** | Géométrie consumer en `Rect`; Arcade dessine hors `Ui`. |
| **Impact** | `Ui` ne peut pas exprimer directement la card grid, mais aucun consumer n'établit qu'il doit l'exprimer. |
| **Structural or local?** | **NON PROMU.** Limite du toolkit ; le problème A3 correspondant est local à Arcade. |
| **Could a targeted change solve it?** | Oui côté Arcade selon son design accepté : fonction pure de layout vers page de `Rect`s. Une primitive GPE ne serait considérée qu'après échec concret de cette voie. |

### Gate 1 verdict

**Gate 1 — PASS.**

**Observed structural problems: NO STRUCTURAL PROBLEM FOUND.**

- Problèmes structurels GPE retenus : **0**.
- Problèmes locaux consumer retenus : **1** (Card UI A3 non implémentée).

Le PASS signifie que les limitations candidates ont été évaluées contre des consumers réels. Il ne signifie ni que l'A3 est terminée, ni que l'architecture future est décidée.

## Gate 2 — highest-impact uncertainties

Trois incertitudes seulement sont retenues. L'ordre reflète qualitativement `Impact × Uncertainty × Cost-to-learn` : priorité aux vérifications peu coûteuses capables de confirmer qu'aucun changement GPE n'est requis ou d'isoler précisément le plus petit manque. Aucun score numérique n'est utilisé.

### U1 — Le contrat A3 complet se mappe-t-il réellement sur les primitives GPE existantes ?

| Champ | Évaluation |
|---|---|
| **uncertainty** | Les affirmations du design Arcade selon lesquelles rendu, coordonnées et input GPE suffisent n'ont pas encore été fermées opération par opération pour l'A3 non implémentée. |
| **why it matters** | C'est la seule pression consumer actuelle susceptible de révéler une primitive générique manquante. |
| **evidence already available** | `ControlMap`, `Input` souris/touch, `VirtualPad`, `Viewport`, `Framebuffer`, `Image::decode_png` et `draw_image_fit` existent ; Arcade assigne layout/hit test à son repository et définit une condition STOP ciblée. |
| **consequence if wrong** | Une dépendance cachée découverte tardivement pourrait bloquer A3 ; inversement, supposer un manque maintenant créerait de la machinerie sans preuve. |
| **decision blocked by it** | Toute décision de modifier GPE pour Arcade, et donc toute promotion de layout/input en problème structurel. |
| **cheapest way to reduce it** | Produire une matrice consumer-only « critère A3 → opération → primitive GPE existante → preuve/test disponible → gap exact ou NONE », limitée à keyboard/gamepad, hover/click, touch, draw/hit-test et small/medium/wide. Pas de prior art ni design de framework. |

**Priorité qualitative : première.** Coût de lecture/traçage faible et information gain maximal : la matrice peut conclure `NONE` ou nommer un manque falsifiable.

### U2 — Quelle règle locale maintient sélection et pagination lors des changements de géométrie/catalogue ?

| Champ | Évaluation |
|---|---|
| **uncertainty** | A3 exige grille responsive et pages, mais les règles de déplacement 2D, changement de page, conservation de sélection et réaction à un catalogue filtré ne sont pas encore spécifiées. |
| **why it matters** | Cette règle détermine si l'index/menu actuel suffit localement, si `ArcadeGameId` doit restaurer la sélection, ou si une pression d'identité/focus générique apparaît réellement. |
| **evidence already available** | `ArcadeGameId` et ordre résolu sont stables ; `MenuState` est ordinal ; le catalogue est résolu avant `ArcadeApp::new`; les layouts small/medium/wide et pagination sont exigés mais non implémentés. |
| **consequence if wrong** | Focus perdu ou carte différente sélectionnée après filtrage, pagination ou changement de largeur ; promotion prématurée possible d'un `UiId` générique. |
| **decision blocked by it** | Statut futur de `state/identity` et `input/focus` : consumer-local suffisant ou besoin GPE démontré. |
| **cheapest way to reduce it** | Écrire une table de transitions Arcade pure sur quelques cas limites : 1 entrée, dernière carte d'une page incomplète, changement du nombre de colonnes, entrée masquée et retour de jeu. Comparer explicitement sélection par index et restauration par `ArcadeGameId`, sans API GPE proposée. |

**Priorité qualitative : deuxième.** Impact UX et déterminisme élevés, coût documentaire faible, mais la question reste d'abord une politique Arcade.

### U3 — Les traces souris/tactile définies pour une carte sont-elles non ambiguës sur native et Web ?

| Champ | Évaluation |
|---|---|
| **uncertainty** | Le design nomme hover/click et touch launch/select, sans fixer encore les transitions press/release/move/cancel, la sortie de hit target, ni l'interaction avec les release gates de lancement/retour. |
| **why it matters** | Une mauvaise règle peut lancer deux fois, lancer après drag/cancel ou laisser l'entrée fuir vers le jeu ; les différences de séquencement Web doivent rester visibles. |
| **evidence already available** | `Input` expose position/boutons souris et touches ordonnées ; `VirtualPad` caractérise move/cancel ; Arcade possède déjà des release gates et `Viewport` fait le mapping de coordonnées. Aucun test A3 card n'existe encore. |
| **consequence if wrong** | Interaction incohérente native/Web ou activation accidentelle, même si le layout est correct. |
| **decision blocked by it** | Confirmation que l'input actuel suffit ; éventuel besoin minimal d'une primitive/event générique. |
| **cheapest way to reduce it** | Définir des traces consumer attendues pour mouse press/release inside/outside, drag out/in, touch start/move/end/cancel et retour de jeu ; mapper chaque observation aux APIs actuelles et marquer les hypothèses Web à valider ultérieurement. |

**Priorité qualitative : troisième.** Information utile et coût limité, mais dépend des règles de sélection/pagination clarifiées par U2.

### Gate 2 verdict

**Gate 2 — PASS.**

Le budget d'incertitude est limité à **3** questions consumer-backed. Toutes peuvent d'abord être réduites par une tranche documentaire étroite centrée sur Arcade ; aucune ne justifie encore prior art général, architecture ou implémentation moteur.

## Research scope reduction

Les classifications suivantes portent sur la **prochaine tranche de recherche**, pas sur une architecture finale.

| Thème | Classement | Justification bornée |
|---|---|---|
| `state/identity` | **PRESSURE CHECK ONLY** | U2 doit vérifier si `ArcadeGameId` + état local suffisent. Aucun consumer actuel ne requiert un `UiId` générique. |
| `layout` | **REQUIRED NOW** | Seulement pour fermer le contrat consumer A3 small/medium/wide vers page de `Rect`s. Pas de recherche Flex/Grid/CSS ni de layout engine générique. |
| `input/focus` | **REQUIRED NOW** | U2/U3 : navigation carte, sélection, hover/click/touch et release gates. Rester au niveau des traces Arcade et APIs GPE existantes. |
| `frame transaction` | **PRESSURE CHECK ONLY** | Vérifier uniquement que les traces A3 peuvent être ordonnées sans activation/fuite ; ne pas concevoir un modèle de frame général. |
| `event routing` | **NO CURRENT NEED** | A3 utilise une hit target carte plate ; aucun arbre nested/capture/bubble consumer-backed. |
| `modularity` | **NO CURRENT NEED** | Aucun problème structurel ni extraction de crate démontré ; Arcade possède déjà ses concepts produit. |
| `testing` | **REQUIRED NOW** | La fermeture des trois incertitudes doit nommer les tests headless minimaux et les hypothèses native/Web, sans les implémenter ici. |
| `performance` | **PRESSURE CHECK ONLY** | Catalogue curate et pagination ; aucune mesure ni régression observée. Relever seulement les opérations dont le coût devrait être mesuré si le catalogue change d'échelle. |
| `styling` | **PRESSURE CHECK ONLY** | Artwork/card presentation est une pression réelle mais explicitement possédée par Arcade ; vérifier que le rendu existant suffit, sans système de styles générique. |
| `game feedback` | **NO CURRENT NEED** | Aucun critère A3 actuel n'exige audio/haptique/animation. `AudioBus::Ui` existe mais n'est pas une justification. |
| `text/i18n` | **NO CURRENT NEED** | Les titres actuels sont des chaînes catalogue simples ; aucun besoin shaping/BiDi actuel observé. |
| `accessibility` | **DEFER** | Aucun consumer actuel ne justifie une étude avancée. La visibilité de sélection reste une propriété locale sous `input/focus`. |
| `markup` | **DEFER** | Aucun frontend déclaratif consumer-backed et aucun dead-end coûteux démontré. |
| `SVG` | **DEFER** | Arcade a accepté PNG RGBA embarqué ; aucun besoin vector runtime. |
| `world-space UI` | **DEFER** | Aucun consumer inspecté. |
| `inspector` | **DEFER** | Aucun besoin structurel actuel ; des tests/dumps purs suffisent pour la tranche suivante. |
| `hot reload` | **DEFER** | Aucun consumer ni dead-end démontré. |
| `advanced animation` | **DEFER** | Aucun critère A3 actuel ne l'exige. |

### Synthèse par classe

**REQUIRED NOW**

- `layout` — uniquement contrat Arcade de page de `Rect`s ;
- `input/focus` — uniquement navigation et traces pointer/touch Arcade ;
- `testing` — preuves headless et pression native/Web de ces mécanismes.

**PRESSURE CHECK ONLY**

- `state/identity` ;
- `frame transaction` ;
- `performance` ;
- `styling`.

**DEFER**

- `accessibility` avancée ;
- `markup` ;
- `SVG` ;
- `world-space UI` ;
- `inspector` ;
- `hot reload` ;
- `advanced animation`.

**NO CURRENT NEED**

- `event routing` ;
- `modularity` ;
- `game feedback` ;
- `text/i18n` avancé.

## Ce qui MUST NOT être recherché ensuite

La prochaine tranche ne doit pas :

- lancer un catalogue de prior art UI ;
- comparer Architecture 0/A/B/C ;
- concevoir un kernel UI, `UiTree`, `UiId` générique ou retained tree ;
- étudier Flexbox, Grid CSS, Taffy, Yoga ou un layout engine général ;
- concevoir capture/bubble, propagation DOM ou un event router ;
- produire un dependency DAG ou recommander une crate/feature ;
- étudier markup, XML/HTML, SVG, inspector, hot reload, world-space UI, animation ou accessibilité avancée ;
- sélectionner ou concevoir un MFE ;
- implémenter A3, un test, un widget ou une modification GPE ;
- élargir l'audit à d'autres consumers sans gap précis issu d'U1–U3.

## Prochaine tranche de recherche recommandée — et seulement celle-ci

**Arcade A3 contract closure, documentaire et consumer-first.**

Produire une seule analyse ciblée qui :

1. mappe chaque critère A3 aux primitives GPE existantes et conclut `SUPPORTED`, `ASSUMPTION TO VALIDATE` ou `EXACT GAP` ;
2. fixe les cas de transition de sélection/pagination nécessaires pour décider index local versus restauration par `ArcadeGameId` ;
3. fixe les traces souris/tactile et les hypothèses native/Web à tester ;
4. s'arrête immédiatement si aucun gap générique n'est trouvé ;
5. si un gap existe, le formule comme la plus petite opération manquante, sans proposer encore d'architecture.

Cette tranche maximise l'information gain à faible coût et protège les deux résultats légitimes : `aucun changement GPE nécessaire` ou `un manque générique précis est démontré`.

## STOP

Gate 1 et Gate 2 sont passés. La recherche est réduite, mais aucune recherche suivante n'est exécutée dans ce document.

**STOP AVANT PRIOR ART ET ARCHITECTURE.**
