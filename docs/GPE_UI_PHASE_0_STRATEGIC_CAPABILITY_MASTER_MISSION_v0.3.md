# GPE.UI — PHASE 0
# STRATEGIC CAPABILITY / ARCHITECTURE / MODULARITY / MIGRATION MASTER MISSION v0.3

Repository principal :

`https://github.com/gotoo77/gotoo-pixel-engine`

Projet consommateur immédiatement visible :

`https://github.com/gotoo77/gpe_arcade`

Baseline factuelle déjà acquise :

```text
GPE audited baseline:
6ff4f8baddae269baa6a7d182f0ba0c9d985f886

Gate 0 checkpoint:
27af0fd096878ea84da5f41716c7dde7a031a6b7

Gate 1 / Gate 2 checkpoint:
c1a3646199705525d1ae8caa03b67f19a556cdbd
```

Ces checkpoints sont des **preuves de départ**, pas des contraintes destinées à réduire artificiellement la mission.

---

# 0. STATUT ET INTENTION

Cette version `v0.3` reprend le moteur intellectuel du master prompt `v0.1` :

> **explorer sérieusement le design space d'un excellent système UI pour GPE, puis réduire l'architecture par preuves, coûts, ergonomie, migration et expérimentation.**

Elle conserve les durcissements utiles acquis ensuite :

- `NONE FOUND` est une conclusion valide ;
- les cardinalités sont des maxima, jamais des quotas à remplir ;
- toute preuve code doit avoir une provenance ;
- aucun pseudo-score numérique de risque/coût sans mesure ;
- Architecture 0 doit être considérée sérieusement ;
- Rust ergonomics ne peut être réellement validée que par un MFE compilé ;
- la migration de l'existant est obligatoire ;
- STOP absolu avant implémentation ;
- les findings adversariaux ne doivent jamais être inventés ;
- Native/Web sont des pressions réelles mais Phase 0 n'est pas une campagne plateforme ;
- le prior art doit dater/versionner les faits susceptibles d'évoluer.

La correction fondamentale du `v0.3` est :

> **Une capacité peut être architecturalement légitime soit parce qu'un consumer réel la demande, soit parce qu'elle constitue une exigence stratégique explicite du produit GPE.UI.**

Classifier toute capacité importante :

```text
OBSERVED REQUIREMENT
→ démontré par un consumer réel

STRATEGIC REQUIREMENT
→ explicitement désiré pour augmenter les capacités de GPE

SPECULATIVE FEATURE
→ ni observé ni stratégique
```

Les deux premières catégories peuvent influencer l'architecture.

La troisième ne le peut pas sans justification supplémentaire.

---

# 1. MISSION

Conduire une **Phase 0 de recherche, conception, architecture, modularité, testabilité et migration**, sans implémentation, pour un futur sous-système provisoirement nommé :

> **GPE.UI**

L'objectif n'est PAS seulement de corriger des défauts existants.

L'objectif est :

> **concevoir le meilleur système UI modulaire raisonnablement envisageable pour augmenter volontairement le pouvoir d'expression graphique et interactif de GPE, puis réduire cette vision par architecture, coût, ergonomie, migration et expérimentation.**

GPE.UI doit être étudié comme un possible composant phare de GPE.

Ambitions stratégiques :

```text
pleasant to use
small-consumer friendly
modular
pay-for-what-you-use
composable
responsive
pixel-aware
multi-input
game-oriented
highly customizable
custom-widget friendly
headless-testable
deterministic where reasonable
native/Web compatible
migratable from existing src/ui
prepared for future declarative authoring without becoming a browser
```

Ces ambitions sont des **STRATEGIC REQUIREMENTS**.

Elles ne nécessitent pas qu'un consumer actuel soit déjà bloqué pour être étudiées.

---

# 2. RÈGLE ABSOLUE

Cette mission est exclusivement :

```text
RESEARCH
AUDIT
DESIGN SPACE EXPLORATION
ARCHITECTURE
MODULARITY DESIGN
MIGRATION DESIGN
TEST STRATEGY
ADVERSARIAL REVIEW
SYNTHESIS
MFE DESIGN ONLY
```

Elle NE DOIT PAS :

```text
écrire du code runtime
créer une nouvelle crate
modifier une API publique GPE
ajouter une dependency runtime
implémenter un widget
implémenter un layout engine
implémenter Flexbox/Grid
implémenter un parser XML/HTML
implémenter SVG
implémenter des cards Arcade
migrer l'UI existante
déprécier/supprimer une API
modifier un jeu
```

Des pseudo-APIs courtes sont autorisées uniquement pour :

```text
comparer
falsifier
tester l'ergonomie conceptuelle
```

# ABSOLUTE STOP BEFORE IMPLEMENTATION

---

# 3. NORTH STAR

La mission doit répondre à deux questions simultanément.

## Q1 — Existing-system question

> Qu'est-ce que GPE possède déjà et qu'il serait absurde de réinventer ?

## Q2 — Strategic-capability question

> Quel système UI augmenterait significativement les capacités de GPE tout en restant modulaire, contrôlable, testable et agréable ?

Une réponse du type :

```text
"Les consumers actuels peuvent déjà tout bricoler localement"
```

ne répond PAS à Q2.

Inversement :

```text
"Construisons un mini navigateur parce que ce serait puissant"
```

ne répond pas correctement aux contraintes de coût et de contrôle.

---

# 4. EXIGENCES STRATÉGIQUES EXPLICITES

Les capacités suivantes ont le droit d'être étudiées même sans consumer bloqué aujourd'hui :

```text
modular kernel
small-consumer minimum
composition
responsive layout
pixel-aware geometry
custom widgets
style/theme extensibility
headless testing
multi-input
game-oriented feedback hooks
deterministic behavior where practical
clear migration path
future declarative frontend compatibility
```

Elles sont **STRATEGIC REQUIREMENTS**.

Les capacités suivantes ne sont PAS automatiquement légitimes :

```text
full HTML5
full CSS
JavaScript
browser DOM compatibility
full SVG 2
arbitrary scripting
remote UI content
complex reactive runtime
full browser-grade accessibility stack
full browser-grade text engine
```

---

# 5. PRINCIPES CANDIDATS À ATTAQUER

Pour chaque principe :

```text
KEEP
REVISE
REJECT
DEFER
```

## P1 — Composition over specialization

Les widgets complexes devraient autant que possible être composés de primitives.

## P2 — Policy != mechanism

Le kernel fournit les mécanismes ; le consumer possède ses politiques produit/gameplay.

## P3 — Semantic input

L'UI devrait pouvoir raisonner en intentions :

```text
Confirm
Cancel
Up
Down
Left
Right
Next
Previous
```

plutôt qu'être directement câblée aux touches physiques.

## P4 — Semantic output

L'UI devrait pouvoir produire des événements/actions sémantiques plutôt que cacher des appels gameplay opaques.

## P5 — One source of truth

L'état gameplay reste dans le jeu/application.

## P6 — Deterministic by default

À state, dimensions et inputs identiques, layout/focus/events/render decisions devraient être reproductibles autant que raisonnablement possible.

## P7 — Headless first

Tout ce qui n'exige pas réellement GPU/fenêtre doit pouvoir être testé sans GPU/fenêtre.

## P8 — Pay only for what you use

Le petit consumer ne doit pas payer pour les capacités lourdes qu'il n'utilise pas.

## P9 — Rust API is first-class

Un éventuel markup est un frontend, jamais l'autorité interne.

## P10 — Pixel-aware, not pixel-imprisoned

Excellent pour pixel-art sans condamner les usages plus riches.

## P11 — No hidden globals

Pas de global UI manager opaque.

## P12 — Escape hatches are legitimate

Un consumer doit pouvoir produire un composant réellement custom sans forker GPE.UI.

## P13 — Feedback is composable

Audio, haptics et animation ne doivent pas gonfler les widgets fondamentaux.

## P14 — Migration over rewrite

Capitaliser sur `src/ui`.

## P15 — Testability is architecture

Les choix de structure doivent permettre une stratégie de test forte.

## P16 — Strategic capabilities are legitimate requirements

Une capacité explicitement voulue par le projet peut être étudiée avant qu'un consumer ne l'ait bricolée.

## P17 — Capability does not imply kernel machinery

Une exigence stratégique peut être servie au-dessus du kernel, via couche/module optionnel.

---

# 6. ANTI-OBJECTIFS

## Browser accident

À chaque proposition HTML/XML/CSS/SVG demander :

> Sommes-nous en train de réimplémenter un navigateur ?

Présomption défavorable envers :

```text
full HTML compatibility
full CSS cascade
JavaScript
DOM Web API
browser event model copied wholesale
full CSS layout engine
```

## Enterprise framework accident

Smells :

```text
UiManagerFactoryProvider
AbstractWidgetControllerStrategy
GlobalUiServiceLocator
```

## Feature soup

Ne pas jeter layout/widgets/themes/markup/SVG/animation/inspector/accessibility dans un kernel monolithique.

## Premature crate explosion

Ne pas faire :

```text
1 concept = 1 crate
```

par principe.

## Existing-consumer prison

Interdit de conclure :

```text
"personne n'en a besoin aujourd'hui donc cette capacité n'a pas à être étudiée"
```

si elle est un STRATEGIC REQUIREMENT.

## Vision-only architecture

Aucune ambition stratégique ne dispense de :

```text
cost
modularity
migration
ergonomics
testability
failure modes
```

---

# 7. BASELINE FACTUELLE À PRÉSERVER

Les travaux Gate 0/1/2 ont établi que GPE dispose déjà de briques utiles autour de :

```text
Ui
UiState
UiTheme
UiResponse
RepeatState
MenuState
PauseGame
VirtualPad
button
toggle
select
slider
tabs
scroll
focus/hover/capture
ControlMap
ActionId
Framebuffer
ImageFit
ImageFilter
```

Limitations observées :

```text
ordinal identity limitations
generic Ui reads physical keyboard/mouse directly
root layout essentially vertical-column oriented
```

Mais les preuves acquises disent aussi :

```text
no structural failure of src/ui demonstrated
no new crate required by current consumers
no UiTree required by current consumers
no generic UiId required by current consumers
Arcade A3 can currently be implemented without new GPE.UI architecture
```

Distinction obligatoire :

```text
A3 is NOT proof that GPE.UI is mandatory.

A3 is ALSO NOT proof that strategic GPE.UI R&D is unnecessary.
```

---

# 8. AUDIT / CAPITALISATION DE L'EXISTANT

Inspecter le repository réel.

Lire intégralement les fichiers pertinents présents au moment de l'exécution, notamment si disponibles :

```text
Cargo.toml
src/lib.rs
src/ui/mod.rs
src/ui/toolkit.rs
src/ui/pause.rs
src/ui/virtual_pad.rs
src/ui/ordinal_identity_tests.rs
src/ui/tabs_contract_tests.rs
src/control.rs
src/input.rs
src/framebuffer.rs
src/bitmap_font.rs
src/image.rs
src/image_fit.rs
src/audio.rs
src/platform.rs
src/viewport.rs
```

Pour chaque brique UI :

```text
symbol
source
responsibility
current consumers
strengths
limitations
state ownership
input coupling
render coupling
test coverage
migration potential
```

Classer :

```text
KEEP
GENERALIZE
MOVE
WRAP
RETHINK
DEPRECATE LATER
DELETE LATER
UNRESOLVED
```

Toute preuve code doit indiquer :

```text
repository
exact ref / SHA
file
symbol/test/section
```

---

# 9. CONSUMER / DESIGN PROBES

Les consumers servent à tester les architectures, PAS à décider si GPE.UI a le droit d'exister.

## Probe A — Tiny UI

```text
Panel
Text
Button
```

Teste :

```text
small-consumer cost
boilerplate
minimal dependencies
```

## Probe B — Pause

```text
overlay
buttons
modal behavior
focus
resume/settings/quit
keyboard/gamepad/touch
```

## Probe C — Settings

```text
tabs
toggle
select
slider/gauge
repeat
scroll
live apply
```

## Probe D — Arcade

```text
card grid
responsive layout
pagination
artwork
keyboard
gamepad
mouse
touch
```

## Probe E — HUD

```text
anchored labels
gauges
dynamic values
status
```

## Probe F — Highly Custom Game UI

```text
Diablo-like inventory
health orb
radial spell selector
rune wheel
weird pixel-art menu
```

## Probe G — Game Feel

```text
Button
+ sound
+ haptic
+ animation
+ semantic action
```

## Probe H — Debug/Probe UI

```text
dense values
controls
diagnostic text
layout dump
```

---

# 10. PRIOR ART — DESIGN-SPACE EXPLORATION

Le prior art est autorisé à explorer au-delà des besoins actuels, car la mission est stratégique.

Examiner au minimum des représentants pertinents de :

## Immediate UI

```text
Dear ImGui
egui
microui
```

## Retained / declarative

```text
Flutter
Slint
Iced
Qt/QML
React-like models
```

## Game UI

```text
Unity UI / UI Toolkit
Godot Control
Bevy UI
```

## Layout

```text
Flexbox
CSS Grid
Taffy
Yoga
constraint systems
```

## Declarative authoring

```text
HTML/CSS
XAML
QML
Slint markup
Godot scenes/resources
```

## Vector / SVG

Étudier stratégies :

```text
full runtime
subset
build-time
raster cache
```

Pour chaque prior art :

```text
problem solved
state model
layout model
input model
style model
render model
test model
modularity
cost characteristics
what GPE should learn
what GPE should not copy
```

Pour les faits susceptibles d'évoluer :

```text
date
version
source
```

Distinguer :

```text
FACT
INFERENCE
RECOMMENDATION
```

---

# 11. KERNEL / STATE / IDENTITY / LIFECYCLE

Comparer sérieusement :

```text
A — immediate
B — retained
C — hybrid
D — evolve current model
```

Étudier :

```text
widget identity
ordinal identity
stable identity
state ownership
focus
pointer capture
scroll state
animation state
tree lifetime
temporary tree
persistent tree
reconciliation
consumer-owned state
framework-owned state
```

Question :

> Quel est le plus petit modèle interne capable de supporter composition, layout, focus, testing, custom widgets et future authoring sans devenir un DOM lourd ?

Ne présume ni `UiTree`, ni son absence.

---

# 12. FRAME TRANSACTION MODEL

Définir pour chaque architecture candidate l'ordre temporel d'une frame UI.

Exemple non imposé :

```text
input snapshot
→ describe UI
→ measure
→ layout
→ hit-test
→ interaction/focus resolution
→ semantic actions
→ gameplay mutation
→ paint
→ render
```

Documenter :

```text
when state mutates
when gameplay mutates
when focus mutates
when actions emit
when paint observes state
one-frame latency risks
borrow implications
```

Le modèle doit être compatible avec :

```text
Rust ergonomics
headless tests
determinism
modal UI
```

---

# 13. EVENT / RESPONSE MODEL

Comparer :

```text
immediate returned response
callbacks/closures
semantic events
semantic actions
command queue
direct mutation
```

Étudier :

```text
hit-test winner
consumption
parent/child propagation
modal interception
pointer capture
drag
```

Ne pas copier le DOM par défaut.

---

# 14. MODULARITY — AXE STRATÉGIQUE CENTRAL

Principe :

> **Un petit consumer doit pouvoir n'utiliser qu'un petit subset de GPE.UI.**

Étudier quatre formes :

## M0 — module interne GPE

```text
gpe::ui
```

## M1 — crate dédiée

```text
gpe-ui
```

## M2 — crate centrale + capacités optionnelles

```text
gpe-ui
+ optional markup
+ optional svg
+ optional inspector
```

## M3 — architecture multi-crates

Exemple conceptuel uniquement :

```text
gpe-ui-core
gpe-ui-layout
gpe-ui-widgets
gpe-ui-feedback
gpe-ui-markup
gpe-ui-svg
gpe-ui-inspector
```

Aucune forme n'est imposée.

Pour chaque frontière :

```text
dependency boundary
ownership boundary
compile boundary
binary boundary
API boundary
versioning boundary
maintenance cost
```

---

# 15. DEPENDENCY DAG

Produire pour chaque architecture sérieuse un DAG conceptuel.

Pour chaque module/crate :

```text
depends on
must not depend on
public types crossing boundary
optional dependencies
reason
```

Détecter :

```text
cycles
wrong ownership
artificial extraction
feature coupling
crate explosion
```

---

# 16. PAY-FOR-WHAT-YOU-USE MODEL

Étudier :

```text
dependencies
compile time
binary size
WASM size
runtime state
allocations
API surface
mental model
```

Pour chaque capacité :

```text
kernel
layout
widgets
styling
feedback
animation
markup
SVG
inspector
```

Toute valeur de coût doit être :

```text
KNOWN
SUPPORTED ESTIMATE
HYPOTHESIS
UNKNOWN
```

Aucun pseudo-score gratuit.

---

# 17. SMALL-CONSUMER TEST

Pour chaque architecture :

> Que doit comprendre/importer/compiler un consumer voulant seulement :

```text
Panel
Text
Button
```

Produire :

```text
required modules
required concepts
state required
boilerplate
dependencies
optional features pulled transitively
```

---

# 18. LAYOUT / RESPONSIVE / PIXEL GEOMETRY

Responsive layout est un **STRATEGIC REQUIREMENT**.

Étudier :

```text
constraints
min/max/preferred
intrinsic size
row
column
stack
grid
grow
shrink
wrap
padding
gap
align
justify
anchors
absolute escape hatch
clipping
overflow
scroll
pagination
safe area
```

Comparer :

```text
simple custom constraints
Flex-like subset
Grid-like subset
hybrid
external layout engine
```

Ne pas copier CSS par réflexe.

Pixel concerns :

```text
integer coordinates
deterministic rounding
pixel snapping
nearest-neighbor imagery
integer scaling
logical framebuffer
physical window
HiDPI
```

Question :

> Comment obtenir un responsive puissant sans perdre le contrôle pixel-perfect ?

---

# 19. INPUT / FOCUS / MULTIMODALITY

Multi-input est un **STRATEGIC REQUIREMENT**.

Étudier :

```text
keyboard
gamepad
mouse
touch
```

Concepts :

```text
focus
hover
pressed
activated
disabled
pointer capture
drag
scroll
focus scope
focus restoration
modal focus
spatial navigation
linear navigation
```

Examiner comment capitaliser sur :

```text
ActionId
ControlMap
VirtualPad
Input
```

---

# 20. GAME-SPECIFIC INTEGRATION

Game-oriented feedback est un **STRATEGIC REQUIREMENT**.

Étudier :

```text
semantic actions
sound feedback
haptic feedback
controller glyphs
repeat
hold-to-confirm
animation
transition requests
pause
HUD
```

Éviter le widget monolithique :

```text
Button {
    sound,
    haptic,
    scene,
    callback,
    animation,
    ...
}
```

Étudier la séparation :

```text
semantics
behavior
style
feedback
```

sans imposer cette forme.

---

# 21. STYLING / THEMES / CUSTOMIZATION

High customization est un **STRATEGIC REQUIREMENT**.

Étudier :

```text
theme
design tokens
style class
local overrides
stateful style
nine-slice
sprite-backed widgets
custom paint
custom widgets
custom layout nodes
```

Question :

> Comment offrir une customisation quasi illimitée sans 300 setters ni mini-CSS incontrôlable ?

---

# 22. CUSTOM WIDGET EXTENSION MODEL

Définir les responsabilités minimales nécessaires à un custom component.

Candidats :

```text
measure
layout
interact
paint
semantics
```

Tester :

```text
ArcadeGameCard
HealthOrb
InventorySlot
RadialSelector
DebugOscilloscope
```

---

# 23. TEXT / LOCALIZATION / ACCESSIBILITY

Éviter les dead-ends flagrants sans promettre un navigateur.

Étudier :

```text
measurement
wrapping
ellipsis
multiline
Unicode
localization expansion
font fallback
CJK
RTL/BiDi
text shaping
semantic role
accessible label
focus visibility
reduced motion
touch target sizing
```

Classer :

```text
REQUIRED FOUNDATION
OPTIONAL MODULE
LATER
NON-GOAL
```

---

# 24. PERFORMANCE / MEMORY / DETERMINISM

Étudier :

```text
tree rebuild
layout cost
allocations/frame
String churn
dynamic dispatch
cache invalidation
dirty layout
large lists
virtualization
animation
```

Définir les mesures futures :

```text
allocations/frame
layout time
interaction time
paint time
memory
binary delta
WASM delta
compile delta
```

Ne jamais inventer les valeurs.

---

# 25. TEST ARCHITECTURE — FIRST CLASS

Objectif :

> **un bug UI corrigé doit pouvoir devenir un test reproductible autant que possible.**

Étudier :

```text
unit tests
property-based tests
golden layout tests
render snapshots
input traces
replay
cross-frontend equivalence
fuzzing where justified
```

Headless est un **STRATEGIC REQUIREMENT**.

---

# 26. OBSERVABILITY / INSPECTOR

Étudier un noyau pouvant produire :

```text
state dump
layout dump
focus dump
event trace
effect trace
```

Inspector graphique :

```text
OPTIONAL CAPABILITY CANDIDATE
```

Ne pas construire le kernel autour de lui sans nécessité.

---

# 27. DECLARATIVE MARKUP — EXPLORATION AUTORISÉE

Markup n'est PAS automatiquement `DEFER`.

C'est une capacité candidate stratégique d'authoring.

Comparer :

```text
no markup
custom DSL
XML
HTML-like subset
RON
TOML
JSON
QML-like
```

Principe candidat :

```text
Rust API ──────┐
               ├──> SAME INTERNAL UI MODEL
Markup ────────┘
```

Règles :

```text
NO JavaScript
NO arbitrary code execution
NO browser compatibility promise
semantic action IDs only
Rust ergonomics wins
```

Questions :

```text
Does markup justify stable IDs?
Does it require retained state?
Can it remain a frontend?
Can it be optional?
Can it be build-time?
Can it hot reload later?
```

---

# 28. SVG / VECTOR — EXPLORATION AUTORISÉE

Comparer :

```text
no SVG
build-time SVG -> raster
runtime rasterization
subset SVG
full external library
```

Étudier :

```text
icons
logos
ornaments
controller glyphs
pixel snapping
palette conversion
raster cache
native/Web
dependency cost
```

---

# 29. ANIMATION

Animation légère peut être une capacité stratégique de game feel.

Étudier :

```text
opacity
position
scale
color
clip
duration
delay
easing
```

Ne pas construire After Effects.

Tester l'optionalité.

---

# 30. DATA FLOW

Étudier :

```text
pull state
responses
events
bindings
observable state
derived values
```

Règle :

> GPE.UI ne devient pas propriétaire de l'état gameplay.

---

# 31. WORLD-SPACE UI

Étudier comme compatibility pressure :

```text
nameplate
health bar
interaction prompt
floating damage
speech bubble
```

Question :

> Le modèle interne empêche-t-il inutilement de rendre un subtree UI dans une autre surface/transform ?

---

# 32. AUTHORING / HOT RELOAD

Étudier :

```text
edit
validate
preview
reload
source locations
diagnostics
```

Hot reload n'est pas nécessairement v1.

---

# 33. DIAGNOSTICS / VERSIONING / SECURITY

Étudier :

```text
public API evolution
semver
deprecation
feature flags
markup schema version
style version
strict parsing
unknown fields
migration
```

Sécurité :

```text
no scripting
no arbitrary code execution
bounded parser behavior
resource limits where relevant
```

---

# 34. ARCHITECTURE 0 — CONCURRENT SÉRIEUX, PAS GAGNANT PAR DÉFAUT

Architecture 0 :

```text
evolve existing src/ui
without a distinct new subsystem/crate
```

Question correcte :

> Peut-on faire évoluer `src/ui` jusqu'à atteindre la vision stratégique GPE.UI sans mauvaise séparation, sans coût inutile et sans bloquer les objectifs de modularité ?

Ce n'est PAS :

> Les consumers actuels fonctionnent-ils déjà ?

---

# 35. ARCHITECTURES A / B / C

Comparer aussi :

## Architecture A — MINIMAL

Petit noyau évolutif, capacités limitées mais stratégiquement cohérentes.

## Architecture B — BALANCED

Système UI modulaire complet mais contenu.

## Architecture C — AMBITIOUS

Préparé explicitement pour authoring déclaratif, extensions avancées et tooling.

Architecture C ne gagne aucun point simplement parce qu'elle sait faire plus.

---

# 36. REQUIRED ARCHITECTURE OUTPUT

Pour chacune :

```text
module/crate boundaries
state/identity
layout
interaction
focus
event/response
styling
feedback
custom widgets
rendering
testing
modularity
dependency DAG
migration
Rust ergonomics
markup relationship
SVG relationship
risks
```

---

# 37. ARCHITECTURE COMPARISON MATRIX

Comparer :

```text
strategic capability coverage
small-game cost
modularity
Rust ergonomics
testability
pixel control
responsive power
multi-input
customization
custom widget quality
game feedback
future authoring
migration risk
runtime cost evidence
dependency cost evidence
API lock-in risk
browser-accident risk
enterprise-framework risk
```

Aucun pseudo-score de coût sans preuve.

---

# 38. RUST API ERGONOMICS

Produire des pseudo-usages pour :

```text
Tiny menu
Settings
Arcade card grid
HUD
Highly custom widget
```

Chaque exemple :

```text
CANDIDATE
NOT VALIDATED BY COMPILATION
```

Verdicts :

```text
CONCEPTUALLY ERGONOMIC
ERGONOMIC RISK
REQUIRES MFE
```

---

# 39. MIGRATION MAP

Pour chaque primitive actuelle :

```text
current symbol
target concept
classification
adapter strategy
migration phase
deprecation timing
removal gate
```

Classification :

```text
KEEP
GENERALIZE
MOVE
WRAP
RETHINK
DEPRECATE LATER
DELETE LATER
UNRESOLVED
```

Jamais :

```text
delete src/ui
rewrite everything
```

---

# 40. ADVERSARIAL REVIEW

Lancer au minimum :

```text
A — Browser Accident
B — Enterprise Framework
C — Small Game
D — Highly Custom Game
E — Rust Ergonomics
F — Migration
G — Test Engineer
H — Performance
I — Native/Web
J — Future Markup
K — Accessibility/I18N
L — API Evolution
M — Strategic Vision Dilution
N — Feature Soup
O — Crate Explosion
```

Pour chaque vrai finding :

```text
finding
severity qualitative
evidence
architectures affected
mitigation
residual risk
```

`NO MATERIAL FINDING` est valide.

---

# 41. STRATEGIC VISION DILUTION ADVERSARY

Cet adversaire est obligatoire.

Il doit détecter toute conclusion du type :

```text
"the current consumer can implement it locally,
therefore the reusable capability has no value"
```

Il doit demander :

> Sommes-nous en train de réduire une mission de capability R&D à une simple politique de refactoring conservateur ?

Il ne doit PAS empêcher la réduction architecturale.

Il doit préserver l'intention stratégique.

---

# 42. ADVERSARIAL REDUCTION

Après exploration complète, classer les concepts :

```text
KEEP IN KERNEL
KEEP AS CORE MODULE
KEEP AS OPTIONAL CAPABILITY
LATER
REJECT
UNKNOWN
```

Pour :

```text
UiTree
UiId
constraints
grid
themes
style classes
event propagation
semantic actions
feedback
audio
haptics
animation
markup
SVG
inspector
world-space
text shaping
accessibility
```

La réduction arrive **après exploration**, pas avant.

---

# 43. MFE SELECTION

Choisir le MFE par une matrice qualitative :

```text
Candidate MFE
Risk addressed
Uncertainty
Cost-to-learn
Information gain
```

Pas de pseudo-score numérique.

Candidats possibles :

```text
tiny composable menu
responsive card grid
stable identity under dynamic structure
semantic multimodal navigation
custom widget escape hatch
headless layout + focus
style/theme composition
```

Le MFE doit maximiser l'information gain sur les hypothèses architecturales les plus risquées.

---

# 44. MFE_001_PROPOSAL

Définir :

```text
hypotheses
why this experiment
scope
non-goals
candidate API
modules
consumer probe
tests
measurements
human runtime gate
failure criteria
rollback boundary
STOP conditions
```

Résultats futurs :

```text
PASS
PASS WITH CONDITIONS
FAIL
```

---

# 45. LIVRABLES

Destination :

```text
docs/ui/phase0-v03/
```

Couverture obligatoire, nombre de fichiers non imposé.

Fichiers recommandés :

```text
README.md
BASELINE_AND_EXISTING_UI.md
STRATEGIC_REQUIREMENTS.md
PRIOR_ART.md
KERNEL_STATE_IDENTITY.md
LAYOUT_RESPONSIVE.md
INPUT_FOCUS_GAME_INTEGRATION.md
STYLING_CUSTOMIZATION.md
MODULARITY_DEPENDENCY_DAG.md
TESTING_OBSERVABILITY.md
DECLARATIVE_MARKUP_SVG_AUTHORING.md
ARCHITECTURE_CANDIDATES.md
MIGRATION_MAP.md
ADVERSARIAL_REVIEW.md
SYNTHESIS.md
MFE_001_PROPOSAL.md
```

Fusionner si cela améliore la qualité.

Ne pas produire de documents superficiels pour remplir une checklist.

---

# 46. SYNTHESIS — REQUIRED CONTENT

La synthèse doit contenir :

```text
existing baseline
strategic requirements
observed requirements
speculative features
Architecture 0
Architecture A
Architecture B
Architecture C
recommended direction
recommended modularity
candidate dependency DAG
kernel concepts
optional capabilities
rejected concepts
layout direction
input/focus direction
game integration
styling/customization
test architecture
markup verdict
SVG verdict
migration path
MFE proposal
unresolved risks
```

Markup et SVG :

```text
GO
OPTIONAL
LATER
REJECT
MORE RESEARCH
```

---

# 47. DECISION CONFIDENCE

Pour chaque décision majeure :

```text
EVIDENCE-BACKED
STRATEGY-BACKED
CONSUMER-BACKED
PRIOR-ART-BACKED
HYPOTHESIS ONLY
REQUIRES MFE
UNKNOWN
```

Pas de score global unique.

---

# 48. FAILURE-FIRST REVIEW

Pour la recommandation finale, produire seulement les failure modes réellement trouvés.

Pour chacun :

```text
early signal
impact
mitigation
rollback boundary
revisit trigger
```

`NONE FOUND` reste valide.

---

# 49. NATIVE / WEB PRESSURE

Vérifier :

```text
native
WASM
browser input
touch
binary size
filesystem assumptions
asset embedding
```

Résultats :

```text
CODE-PATH COMPATIBLE
RUNTIME VALIDATION REQUIRED
UNKNOWN
```

Ne jamais confondre build/package avec runtime Web validé.

---

# 50. GIT DISCIPLINE

Créer une branche dédiée :

```text
research/gpe-ui-phase0-v03
```

ou équivalent explicite.

Ne travailler que dans :

```text
docs/ui/phase0-v03/**
```

Ne modifier sous aucun prétexte :

```text
src/**
examples/**
Cargo.toml
Cargo.lock
assets/**
```

Ne pas merge.

Push autorisé après validation documentaire.

---

# 51. VALIDATION

Avant commit :

```bash
git diff --check
git status --short
```

Prouver :

```text
files outside docs/ui/phase0-v03/** modified: NONE
```

Aucune validation Rust obligatoire puisque cette mission ne modifie aucun code.

---

# 52. FINAL REPORT TEMPLATE

```text
GPE.UI PHASE 0 v0.3 RESULT

Baseline:
<exact SHA>

Branch:
<name>

Existing UI baseline:
<summary>

Strategic requirements:
<summary>

Observed requirements:
<summary>

Speculative features rejected/deferred:
<summary>

Architecture 0:
<summary>

Architecture A:
<summary>

Architecture B:
<summary>

Architecture C:
<summary>

Recommended direction:
<0 / A / B / C / HYBRID / MORE RESEARCH>

Why:
<concise evidence>

Recommended kernel:
<concepts>

Recommended optional capabilities:
<concepts>

Small-consumer minimum:
<summary>

Modularity:
<summary>

Dependency DAG:
<summary>

Layout:
<summary>

Input/focus:
<summary>

Game integration:
<summary>

Styling/customization:
<summary>

Testing/headless:
<summary>

Markup:
<GO / OPTIONAL / LATER / REJECT / MORE RESEARCH>

SVG:
<GO / OPTIONAL / LATER / REJECT / MORE RESEARCH>

Migration:
<summary>

MFE-001:
<summary>

Decision confidence:
- <decision>: <classifications>

Unresolved risks:
<only real risks>

Files created:
<list>

Files outside docs/ui/phase0-v03 modified:
NONE

Commits:
<list>

Push:
<status>

PHASE 0 v0.3 STATUS:
COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW
```

ou :

```text
INCOMPLETE — BLOCKED
```

si une vraie insuffisance de preuve empêche la synthèse.

---

# 53. FINAL STOP

Après la documentation :

# STOP

Ne :

```text
code rien
crée aucune crate
ajoute aucune dependency
modifie aucun module UI
migre aucun consumer
implémente aucun widget
implémente aucun layout
implémente aucun parser
implémente aucun SVG
implémente aucun MFE
```

La prochaine étape est :

> **independent adversarial review of Phase 0 v0.3**

---

# 54. FINAL NORTH STAR

La Phase 0 v0.3 doit éviter deux échecs symétriques.

## Failure A — overengineering

```text
build everything because it sounds powerful
```

## Failure B — strategic starvation

```text
build nothing reusable because current consumers can survive without it
```

La bonne question est :

> **Quel système UI réutilisable mérite d'être construit pour augmenter volontairement les capacités de GPE, et quelle est la plus petite architecture cohérente capable de porter cette ambition sans devenir un navigateur, un framework enterprise ou une dépendance lourde imposée à tous les jeux ?**

Le résultat peut conclure :

```text
A compact GPE.UI kernel is justified.
```

ou :

```text
Evolve src/ui, but with a deliberate modular architecture.
```

ou :

```text
A separate gpe-ui crate is justified.
```

ou :

```text
Several strategic capabilities are valuable,
but only a subset belongs in v1.
```

Ce qui n'est plus acceptable est :

```text
"No current consumer is blocked,
therefore the strategic capability should not be designed."
```

La mission est une mission de **capability R&D**, pas seulement de maintenance corrective.
