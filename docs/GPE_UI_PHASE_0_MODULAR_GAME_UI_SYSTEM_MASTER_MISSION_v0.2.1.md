# GPE.UI — PHASE 0
# MODULAR GAME UI SYSTEM — RESEARCH / ARCHITECTURE / MIGRATION MASTER MISSION v0.2.1

Repository principal :

`https://github.com/gotoo77/gotoo-pixel-engine`

Baseline historique observée lors de la préparation initiale du chantier :

`82ba7afa0933e5adeaa7ad5c1238d30e7d957771`

Projet consommateur immédiatement concerné :

`https://github.com/gotoo77/gpe_arcade`

Autres consommateurs réels ou potentiels à considérer s'ils sont disponibles en lecture seule :

- GPE Arcade
- Smart Boy Hero
- Void Canticle
- pause/settings/debug/probes GPE
- futurs jeux GPE

---

# 0. STATUT DU DOCUMENT

Cette version `v0.2.1` remplace `v0.2` comme prompt d'exécution recommandé.

Elle conserve la restructuration méthodologique de `v0.2` et ajoute un micro-hardening issu de sa revue adversariale.

La logique reste :

```text
AUDIT
→ OBSERVED PROBLEMS
→ TOP UNCERTAINTIES
→ MINIMUM NECESSARY RESEARCH
→ ARCHITECTURE 0 / A / B / C
→ ADVERSARIAL REDUCTION
→ RISK-RANKED MFE
```

Les corrections `v0.2.1` sont explicitement :

```text
1. NONE FOUND est un résultat valide pour les shortcomings/problems.
2. Les cardinalités sont des maxima : ne jamais compléter artificiellement une liste.
3. Les repositories consumers peuvent être inspectés à distance en READ ONLY si absents localement.
4. Toute preuve issue du code doit avoir une provenance minimale.
5. Risk × Uncertainty × Information Gain est un cadre de priorisation, pas une formule numérique.
6. La parité native/Web reste une pression légère de validation.
7. Le prior art doit dater/versionner les affirmations susceptibles d'évoluer.
```

Le but de cette Phase 0 n'est PAS :

> concevoir le framework UI le plus complet imaginable.

Le but est :

> **identifier le minimum d'architecture UI dont GPE peut démontrer avoir besoin aujourd'hui, sans créer de dead-end coûteux pour demain.**

---

# 1. MISSION

Conduire une **Phase 0 de recherche et d'architecture**, sans implémentation, autour d'un futur sous-système UI provisoirement nommé :

> **GPE.UI**

Le nom final, la forme de distribution et les frontières de crates/modules/features ne sont PAS décidés.

Cette étude doit répondre en priorité à une question plus fondamentale :

> **GPE.UI mérite-t-il réellement d'exister comme abstraction distincte, ou l'évolution ciblée de `src/ui` suffit-elle ?**

Le système étudié pourrait éventuellement devenir un composant phare de GPE :

- agréable à utiliser ;
- fortement testable ;
- déterministe ;
- pixel-aware ;
- responsive ;
- multi-input ;
- adapté aux jeux ;
- composable ;
- extensible ;
- personnalisable ;
- modulaire ;
- contrôlable en coût et dépendances ;
- compatible avec une évolution future sans sur-anticipation ;
- capable de capitaliser sur les briques UI GPE déjà existantes.

Mais cette ambition est une hypothèse à éprouver, pas une conclusion.

Le premier consommateur visible pressenti est le futur Card UI de `gpe_arcade`.

Cependant :

> **GPE.UI ne doit PAS être conçu spécifiquement pour Arcade.**

Arcade est un pressure-test réel parmi plusieurs.

---

# 2. RÈGLE ABSOLUE

Cette mission est limitée à :

```text
RESEARCH
AUDIT
OBSERVED PROBLEM IDENTIFICATION
UNCERTAINTY REDUCTION
ARCHITECTURE
ADVERSARIAL ANALYSIS
MIGRATION DESIGN
TEST STRATEGY
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

Des pseudo-APIs ou pseudo-types courts sont autorisés uniquement pour rendre un choix falsifiable.

# ABSOLUTE STOP BEFORE IMPLEMENTATION

---

# 3. POSTURE

Tu interviens comme une équipe virtuelle composée de plusieurs regards distincts :

- engine architect ;
- UI framework architect ;
- Rust API/library designer ;
- game UI / UX engineer ;
- rendering/layout engineer ;
- input/focus/accessibility engineer ;
- test architecture engineer ;
- performance engineer ;
- tooling/debugging engineer ;
- migration engineer ;
- adversarial reviewer.

Ne cherche pas à confirmer l'idée initiale.

Les idées suivantes sont des **hypothèses candidates**, pas des conclusions :

```text
GPE.UI comme abstraction distincte
crate dédiée
plusieurs crates
Cargo features
kernel UI minimal
UiTree
retained mode
immediate mode
hybrid mode
stable UiId
style classes
feedback audio/haptique
semantic events/actions
markup XML/HTML-like
SVG
inspector
world-space UI
```

Tu dois être prêt à conclure :

```text
GO
LATER
DEFER
UNNECESSARY
TOO EXPENSIVE
WRONG FOR GPE
ARCHITECTURE 0 WINS
MORE RESEARCH
```

---

# 4. PRINCIPE MÉTHODOLOGIQUE CENTRAL

## 4.1 Information gain over coverage

La Phase 0 doit optimiser :

> **information gain / effort**

et non :

> **coverage completeness**

Il est préférable d'identifier correctement quelques incertitudes structurantes que de produire une couverture artificiellement exhaustive.

## 4.2 No current consumer, no kernel machinery

Règle par défaut :

> **Une capacité sans consommateur réel observable ne doit pas introduire de machinerie dans le kernel.**

Exception :

> Une capacité future peut influencer une décision uniquement si son absence crée de manière démontrable un **architectural dead-end coûteux à corriger plus tard**.

```text
current consumer
→ may justify machinery

demonstrable future dead-end
→ may constrain a decision

mere future possibility
→ DEFER
```

## 4.3 Rust ergonomics wins

Une capacité hypothétique de markup ne doit jamais détériorer l'API Rust sans justification indépendante actuelle.

> **Future markup compatibility may reject a dead-end, but may not introduce kernel machinery by itself.**

## 4.4 Compatibility future != preparation future

Toujours distinguer :

```text
"Cette décision nous enferme-t-elle ?"
```

de :

```text
"Construisons maintenant les abstractions du futur."
```

GPE doit privilégier l'évolutivité, pas l'anticipation spéculative.

## 4.5 No artificial cardinality

Lorsqu'une section demande :

```text
up to N findings / risks / uncertainties
```

`N` est un maximum, jamais une cible de remplissage.

Règle :

> **Do not pad a list to reach the maximum.**

Si seulement 2 ou 3 éléments sont réellement étayés, produire 2 ou 3 éléments.

## 4.6 No pseudo-mathematical confidence

Les expressions telles que :

```text
Impact × Uncertainty × Cost-to-learn
Risk × Uncertainty × Information Gain
```

sont des **cadres de priorisation**, pas des équations.

Ne jamais inventer des scores numériques ou une arithmétique de classement sauf si :

```text
a scale is explicitly defined
AND
each score is evidence-backed
```

Sinon comparer qualitativement et expliquer le raisonnement.

---

# 5. PRINCIPES CANDIDATS À ATTAQUER

Pour chacun produire :

```text
KEEP
REVISE
REJECT
DEFER
```

avec preuve ou justification.

## P1 — Composition over specialization
Les widgets complexes devraient autant que possible être composés de primitives.

## P2 — Policy != mechanism
Le kernel fournit les mécanismes ; le jeu possède les politiques produit/gameplay.

## P3 — Semantic input
L'UI devrait parler en intentions telles que `Confirm`, `Cancel`, `Up`, `Down`, `Left`, `Right`, `Next`, `Previous`, plutôt qu'être câblée directement à des touches physiques.

## P4 — Semantic output
L'UI devrait produire des événements/actions sémantiques plutôt que cacher des appels gameplay arbitraires.

## P5 — One source of truth
L'état gameplay reste autoritaire dans le jeu.

## P6 — Deterministic by default
À inputs, dimensions et state identiques, layout/focus/events/render doivent être reproductibles autant que raisonnablement possible.

## P7 — Headless first
Tout mécanisme qui n'exige pas réellement de framebuffer/GPU doit pouvoir être testé sans fenêtre.

## P8 — Pay only for what you use
Un petit consommateur ne doit pas être forcé d'embarquer mentalement, compile-time ou runtime des capacités lourdes inutilisées.

## P9 — Rust API is first-class
Un éventuel markup est un frontend vers le modèle interne, jamais l'autorité architecturale.

## P10 — Pixel-aware, not pixel-imprisoned
Le système doit être excellent pour le pixel-art sans interdire une UI plus riche.

## P11 — No hidden globals
Éviter le global UI manager magique et l'état implicite difficile à raisonner/tester.

## P12 — Escape hatches are legitimate
Un jeu doit pouvoir écrire un composant/widget/render node custom sans forker le framework.

## P13 — Game feedback is composable
Audio, haptics, animation et autres feedbacks doivent rester composables et optionnels.

## P14 — Migration over rewrite
Les briques existantes doivent être auditées avant suppression/réécriture.

## P15 — Testability is architecture
La capacité à tester l'UI influence le design dès le kernel.

---

# 6. ANTI-OBJECTIFS / GARDES-FOUS

## 6.1 Navigateur accidentel

À chaque proposition HTML/CSS/XML/SVG demander :

> Sommes-nous en train de réimplémenter un navigateur ?

Présomption défavorable envers :

```text
HTML5 arbitraire
CSS cascade complète
DOM Web complet
JavaScript
scripting arbitraire
browser compatibility
layout CSS complet
```

## 6.2 Framework enterprise accidentel

Refuser les abstractions sans consommateurs réels.

Smells :

```text
UiManagerFactoryProvider
AbstractWidgetControllerStrategy
GlobalUiServiceLocator
```

## 6.3 Feature soup

Ne pas empiler dans le kernel :

```text
animations
markup
SVG
themes avancés
inspector
accessibility avancée
world-space UI
```

parce qu'ils sont séduisants.

## 6.4 Premature crate explosion

Ne pas conclure :

```text
1 concept = 1 crate
```

sans frontière réelle de coût ou ownership.

## 6.5 Architecture paper-complete

Interdit de produire une architecture « complète sur papier » simplement parce que toutes les rubriques du prompt existent.

Une section peut légitimement conclure :

```text
DEFER
UNKNOWN
INSUFFICIENT EVIDENCE
NO CURRENT NEED
NONE FOUND
```

---

# 7. BASELINE GIT

Avant toute recherche :

```bash
git status
git branch --show-current
git rev-parse HEAD
git log --oneline -10
```

Documenter :

```text
repository
branch
HEAD exact
date
worktree status
```

La baseline historique `82ba7afa0933e5adeaa7ad5c1238d30e7d957771` n'est PAS un verrou.

Si `main` a avancé :

- enregistrer le nouveau HEAD ;
- inspecter l'état réel ;
- utiliser cet état comme baseline ;
- documenter les écarts pertinents.

---

# 8. GATE 0 — EXISTING SYSTEM UNDERSTOOD

Aucune recherche architecturale générale ne doit commencer avant ce gate.

Inspecter le repository réel.

Au minimum lire intégralement les fichiers actuellement pertinents, dont si présents :

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
src/input.rs ou modules input réels
src/framebuffer.rs
src/bitmap_font.rs
src/image.rs
src/image_fit.rs
src/audio.rs
```

Rechercher tous les usages de :

```text
Ui
UiState
UiTheme
UiResponse
RepeatState
RepeatConfig
MenuState
PauseGame
PauseConfig
VirtualPad
VirtualButton
draw_panel
draw_menu_item
draw_text_centered
standard_menu_controls
menu_*_pressed
```

et toute autre primitive découverte.

L'état initial connu comporte déjà des briques autour de :

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
focus ordinal
mouse interaction
menu helpers
```

Mais cette liste n'est pas une autorité.

## Gate 0 doit produire

### A. Existing responsibility map

Pour chaque brique réelle :

```text
symbol
source
responsibility
consumers
state ownership
input coupling
render coupling
tests
```

### B. Observed strengths

Ce que le système fait déjà correctement.

### C. Shortcomings assessment

Évaluer les limites démontrées par le code ou des consumers réels.

Deux résultats sont explicitement valides :

```text
REAL SHORTCOMINGS IDENTIFIED
```

ou :

```text
NO STRUCTURAL SHORTCOMING FOUND
```

Dans les deux cas, fournir les preuves.

L'agent ne doit jamais inventer un défaut pour faire passer le gate.

### D. Existing primitives already satisfying future needs

Ce qui NE DOIT PAS être réinventé.

## Gate 0 PASS

Le gate passe seulement si :

```text
existing UI responsibilities understood
shortcomings assessed
existing strengths identified
consumer evidence assessed
```

Le PASS n'exige PAS de trouver un défaut.

Si les preuves sont insuffisantes :

```text
STOP
PHASE 0 STATUS = INCOMPLETE — BLOCKED
```

---

# 9. GATE 1 — OBSERVED PROBLEMS

Avant de parler d'architecture, produire les problèmes structurels réellement observés.

Pour chaque problème :

```text
Problem
Evidence
Consumer(s)
Current workaround
Impact
Is it structural or local?
Could a targeted fix solve it?
```

Exemples possibles, uniquement si démontrés :

```text
ordinal identity fragility
hardcoded vertical layout
input coupling
lack of spatial navigation
lack of responsive layout
difficult custom widgets
state reset ergonomics
legacy duplication
```

Ne transforme pas automatiquement une limitation locale en problème de framework.

Conclusion valide si aucune preuve ne justifie un problème structurel :

```text
NO STRUCTURAL PROBLEM FOUND
```

Ce résultat doit être considéré comme informatif et peut renforcer Architecture 0.

Question obligatoire :

> Quel est aujourd'hui le plus petit problème UI structurel réel de GPE que nous devons résoudre ?

Réponse valide :

> Aucun problème structurel démontré à ce stade.

si les preuves l'étayent.

---

# 10. GATE 2 — TOP UNCERTAINTIES / UNCERTAINTY BUDGET

Identifier **jusqu'à 5 incertitudes à plus fort impact**.

Ne jamais compléter artificiellement jusqu'à 5.

Pour chacune :

```text
uncertainty
why it matters
evidence available
consequence if wrong
decision blocked by it
cheapest way to reduce it
```

Utiliser comme cadre de priorisation :

```text
Impact × Uncertainty × Cost-to-learn
```

Ce n'est PAS une formule numérique.

Prioriser les recherches qui maximisent l'information gain.

Exemples possibles :

```text
identity/state model
layout model
semantic navigation
existing Ui migration
frame transaction ordering
```

Ne présume pas que ce sont les bons candidats.

## Gate 2 PASS

Le programme de recherche restant doit être explicitement réduit à ce qui est nécessaire pour les incertitudes réellement retenues.

Si aucune incertitude structurante ne subsiste, le dire explicitement au lieu d'en inventer.

---

# 11. RESEARCH SCOPE REDUCTION

Après Gates 0–2, reclasser les thèmes suivants :

```text
REQUIRED NOW
PRESSURE CHECK ONLY
DEFER
NO CURRENT NEED
```

Thèmes candidats :

```text
state/identity
layout
input/focus
frame transaction
event routing
modularity
testing
performance
styling
game feedback
text/i18n
accessibility
markup
SVG
world-space UI
inspector
hot reload
advanced animation
```

Par défaut :

```text
markup                 DEFER
SVG                    DEFER
world-space UI         DEFER
inspector               DEFER
advanced accessibility DEFER
advanced animation     DEFER
hot reload              DEFER
```

Ils ne peuvent monter en `REQUIRED NOW` que si :

```text
current consumer evidence
OR
demonstrable costly architectural dead-end
```

---

# 12. CONSUMER EVIDENCE RULE

Chaque capacité majeure proposée doit être justifiée par :

```text
at least one currently observable consumer use-case
```

ou être marquée :

```text
SPECULATIVE
```

Exemple acceptable :

```text
stable identity

evidence:
- dynamic Arcade list
- Pause focus restoration
```

Exemple insuffisant :

```text
stable IDs are useful in retained UIs
```

Prior art peut soutenir une décision, mais ne remplace pas la pression consommateur.

## 12.1 Remote consumer inspection

Si un repository consumer nommé n'est pas disponible localement mais est accessible via GitHub :

```text
READ-ONLY remote inspection is authorized
```

Il est interdit de :

```text
modifier
brancher
committer
pousser
ouvrir une PR
```

sur un consumer pendant cette mission.

Pour chaque consumer inspecté, enregistrer :

```text
repository
commit SHA or exact ref inspected
files/symbols inspected
availability: LOCAL / REMOTE
```

## 12.2 Evidence provenance — obligatoire

Toute preuve importante issue du code doit être traçable au minimum par :

```text
repository
commit SHA or exact ref
file
symbol/test/section when applicable
consumer manifestation when applicable
```

Pour une preuve de prior art :

```text
source
version/date when behavior may evolve
fact/inference/recommendation classification
```

Ne jamais présenter une impression non traçable comme un fait observé.

---

# 13. CONSOMMATEURS RÉELS À PRESSURE-TESTER

Étudier selon disponibilité et pertinence :

## C1 — Arcade

```text
cards
grid
responsive
pagination/scroll
keyboard
gamepad
mouse
touch
focus
launch
```

Le Card UI Arcade A3 est conceptuellement en attente de cette étude.

## C2 — Pause UI

```text
overlay
modal behavior
focus
resume
settings
quit
gamepad
touch
focus restoration
```

## C3 — Settings UI

```text
tabs
toggles
selectors
sliders/gauges
repeat
live apply
save/cancel
audio feedback
scroll
```

## C4 — HUD / game UI

```text
anchoring
labels
gauges
status
dynamic values
low overhead
```

## C5 — Debug / Probe UI

```text
dense information
headless dumps
layout visualization
interactive controls
fast authoring
diagnostics
```

L'absence d'un consumer ne doit pas être compensée par de la spéculation.

---

# 14. PRIOR ART — MINIMUM NECESSARY RESEARCH

Le prior art n'est pas un catalogue.

Il doit être piloté par les incertitudes Gates 1–2.

Familles possibles :

```text
Dear ImGui
egui
microui
Flutter
Slint
Iced
Qt/QML
Unity UI / UI Toolkit
Godot Control
Bevy UI
Taffy
Yoga
Flexbox
CSS Grid
```

N'étudier que ce qui aide à trancher une question réelle.

Pour chaque source :

```text
mechanism
problem solved
trade-off
relevance to observed GPE problem
what GPE should learn
what GPE should NOT copy
```

Sources primaires privilégiées.

Distinguer :

```text
FACT
INFERENCE
RECOMMENDATION
```

Pour toute affirmation susceptible d'évoluer avec le projet ou la version étudiée, enregistrer :

```text
version/release when known
source publication/update date when relevant
access/research date
```

Ne pas présenter comme intemporel un comportement qui dépend d'une version actuelle.

---

# 15. COST EVIDENCE DISCIPLINE

Aucune estimation décorative du type :

```text
compile cost = LOW
binary cost = MEDIUM
runtime cost = LOW
```

sans preuve.

Chaque coût doit être classifié :

```text
KNOWN
SUPPORTED ESTIMATE
HYPOTHESIS
UNKNOWN
```

> **A cost without measurement or external evidence MUST be marked UNKNOWN.**

Domaines concernés :

```text
dependency cost
compile cost
binary size
WASM size
runtime state
allocations
layout cost
render cost
maintenance cost
API complexity
```

La Phase 0 peut définir **quoi mesurer plus tard**, sans inventer le résultat.

---

# 16. ARCHITECTURE 0 — OBLIGATOIRE

La synthèse doit comparer une option fondamentale :

# Architecture 0 — EVOLVE EXISTING `src/ui`

```text
existing src/ui
+
targeted improvements
+
no new architectural subsystem
+
no new crate unless later proven necessary
```

Question :

> Les problèmes observés peuvent-ils être résolus par évolution locale cohérente du système actuel ?

Si oui, cette option doit pouvoir gagner.

Aucun biais en faveur d'une nouvelle bibliothèque.

---

# 17. ARCHITECTURES A / B / C

Seulement après Gates 0–2 et recherche minimale.

# Architecture A — MINIMAL SUBSYSTEM

Le plus petit sous-système distinct justifié par les preuves.

# Architecture B — BALANCED

Architecture réutilisable/modulaire sans anticipation lourde.

# Architecture C — AMBITIOUS

Architecture plus préparée à des capacités futures, uniquement si les preuves la justifient.

Architecture C ne gagne aucun point simplement parce qu'elle couvre plus de fonctionnalités.

Toute architecture peut être éliminée tôt si les preuves disponibles suffisent.

---

# 18. QUESTIONS STRUCTURANTES — STATE / IDENTITY / OWNERSHIP

Comparer si nécessaire :

```text
pure immediate
retained
hybrid
```

Étudier seulement au niveau requis :

```text
widget identity
ordinal identity
stable identity
state ownership
focus state
pointer capture
scroll state
tree lifetime
consumer-owned state
framework-owned state
```

Questions :

1. L'identité ordinale actuelle est-elle réellement un problème ?
2. Dans quels consumers ?
3. Stable `UiId` apporte-t-il une valeur démontrable ?
4. Qui possède le focus ?
5. Qui possède le scroll ?
6. Qui possède l'état gameplay ?
7. Un tree persistant est-il nécessaire ?
8. Une description temporaire suffit-elle ?

Ne pas introduire reconciliation/diffing/lifecycle sans besoin prouvé.

---

# 19. FRAME TRANSACTION MODEL — OBLIGATOIRE

Pour chaque architecture candidate encore en lice, définir explicitement l'ordre temporel d'une frame UI.

Exemple candidat, non imposé :

```text
input snapshot
→ UI description/build
→ measure/layout
→ hit-test
→ focus/event resolution
→ semantic actions
→ gameplay mutation
→ render
```

Documenter :

```text
when state may mutate
when gameplay may mutate
when focus changes
when pointer capture changes
when actions are emitted
when rendering sees updated state
whether there is one-frame latency
```

Ce modèle doit être compatible avec :

```text
determinism
Rust borrowing
headless tests
modal behavior
event consumption
```

---

# 20. EVENT ROUTING / CONSUMPTION — OBLIGATOIRE

Définir au minimum :

```text
direct dispatch?
parent propagation?
capture?
bubble?
consumption?
priority?
modal interception?
pointer capture?
```

Il n'est PAS demandé de copier le DOM.

Il est demandé de répondre :

> Qui reçoit quoi, qui peut consommer quoi, et dans quel ordre ?

Tester conceptuellement :

```text
Panel
└ Card
  └ Button
```

et :

```text
Game
→ Pause overlay
→ Settings modal
→ Confirmation modal
```

---

# 21. LAYOUT / RESPONSIVE / PIXEL GEOMETRY

Ce thème est `REQUIRED NOW` seulement si Gate 1 le justifie.

Étudier au niveau nécessaire :

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
alignment
clipping
overflow
scroll
pagination
safe areas
logical framebuffer
integer scaling
```

Comparer seulement les modèles utiles :

```text
simple constraints
Flex-like subset
Grid-like subset
hybrid
```

Ne pas copier CSS par défaut.

Pixel concerns :

```text
integer coordinates
pixel snapping
rounding policy
deterministic rounding
nearest-neighbor assets
text pixel alignment
```

Question :

> Quel minimum résout les consumers réels sans créer de moteur CSS ?

---

# 22. INPUT / FOCUS / MODALITY

Étudier selon besoins observés :

```text
keyboard
gamepad
mouse
touch
```

Concepts possibles :

```text
focus
hover
pressed
active
activated
disabled
pointer capture
drag
scroll
focus scope
focus restoration
modal interception
hit testing
spatial navigation
linear navigation
```

Relation avec :

```text
ActionId
ControlMap
ControlBinding
GamepadButton
Input
VirtualPad
```

Question centrale :

> Input brut, actions UI sémantiques, ou combinaison ?

Navigation gamepad spatiale doit être étudiée seulement si un consumer la réclame réellement.

---

# 23. GAME-SPECIFIC INTEGRATION

Tester au niveau des besoins réels :

```text
semantic UiAction
semantic UiEvent
sound feedback
haptic feedback
controller glyphs
hold-to-confirm
repeat input
pause overlays
HUD
```

Éviter :

```text
Button {
    click_sound,
    haptic,
    scene,
    callback,
    animation,
    ...
}
```

Comparer :

```text
callbacks
responses
events
actions
command queue
```

Critères :

```text
Rust ergonomics
testability
serialization/replay potential
markup neutrality
debuggability
ownership clarity
```

Les feedbacks optionnels ne doivent pas devenir dépendances obligatoires du kernel sans preuve.

---

# 24. STYLING / COMPOSITION / CUSTOM EXTENSION

Étudier seulement le minimum utile aux consumers actuels.

Candidats :

```text
theme
tokens
style class
stateful style
composition
custom paint
custom widgets
nine-slice
sprite-backed controls
```

Question :

> Comment éviter à la fois 200 setters et une mini-CSS ?

Un escape hatch doit être possible pour des widgets comme :

```text
HealthOrb
InventorySlot
ArcadeGameCard
DebugOscilloscope
```

sans modifier le kernel.

---

# 25. TEXT / I18N / ACCESSIBILITY PRESSURE

Par défaut :

```text
advanced text shaping = DEFER
advanced accessibility = DEFER
```

Mais vérifier que les choix de kernel n'introduisent pas de dead-end manifeste.

Pressure tests :

```text
PLAY
CONTINUE
REPRENDRE LA PARTIE
SPIEL FORTSETZEN
```

et au moins une langue hors latin comme pression conceptuelle si cela apporte une information architecturale utile.

Étudier si nécessaire :

```text
measurement
wrapping
multiline
Unicode assumptions
semantic role
accessible label
focus visibility
reduced motion
```

Ne pas promettre :

```text
full CJK
BiDi
screen reader
```

sans recherche/prototype dédiés.

---

# 26. MODULARITY / CRATE-MODULE DEPENDENCY DAG — OBLIGATOIRE

Avant toute recommandation de crate(s), produire :

# CANDIDATE DEPENDENCY DAG

Pour chaque candidat :

```text
crate/module
depends on
must NOT depend on
public types crossing boundary
optional dependencies
reason
```

Inclure selon pertinence :

```text
gpe core
ui
framebuffer
input/control
font/text
image
audio
game feedback
markup
SVG
inspector
```

Le DAG doit détecter :

```text
cycles
artificial extraction
wrong type ownership
feature coupling
crate fragmentation
dependency inversion
```

Comparer :

```text
module only
single gpe-ui crate
gpe-ui + optional heavy crates
multiple feature-gated capabilities
```

Aucune crate n'est recommandée sans raison de coût/ownership/versioning.

---

# 27. SMALL-GAME MINIMUM — OBLIGATOIRE

Pour chaque architecture candidate encore en lice :

> Quel est le minimum qu'un petit jeu doit comprendre/importer/compiler pour afficher `Panel + Text + Button` ?

Produire :

```text
concepts required
modules required
dependencies required
state required
boilerplate
```

L'adversaire `Small Game` agit ici immédiatement.

Une architecture qui échoue ce test peut être éliminée avant la suite.

---

# 28. RUST ERGONOMICS — CONCEPTUAL ONLY

Produire des pseudo-usages courts pour :

```text
tiny pause menu
settings screen
Arcade list/grid
custom game widget
```

Évaluer :

```text
borrow shape
state wiring
boilerplate
imports
error handling
customization path
```

Mais le verdict Phase 0 doit être :

```text
CONCEPTUALLY ERGONOMIC
```

ou :

```text
ERGONOMIC RISK
```

Jamais :

```text
RUST ERGONOMIC
```

Cette dernière propriété ne peut être validée que par un MFE compilé.

L'adversaire `Rust Ergonomics` doit agir pendant chaque proposition, pas seulement à la fin.

---

# 29. MIGRATION PRESSURE — CONTINUE

L'adversaire `Migration` agit pendant toute la conception.

Pour chaque nouveau concept demander :

```text
How does existing Ui migrate?
Can behavior remain constant?
Adapter possible?
Temporary coexistence?
Breaking change?
Removal gate?
```

Aucune solution ne doit être recommandée si elle nécessite implicitement :

```text
rm -rf src/ui
rewrite everything
```

---

# 30. MARKUP — DEFER BY DEFAULT

Première question :

> Existe-t-il un consumer actuel nécessitant un frontend déclaratif ?

Si non :

```text
MARKUP = DEFERRED CAPABILITY PRESSURE
```

Ne répondre alors qu'à :

```text
Does current kernel choice create a costly future dead-end?
If yes, how?
If no, stop.
```

Si une recherche plus poussée est réellement justifiée, comparer brièvement :

```text
custom DSL
XML
HTML-like subset
RON/TOML/JSON
no markup
```

Règles :

```text
Rust ergonomics wins
No JavaScript
No arbitrary code execution
Semantic action IDs only
Markup is frontend, never kernel authority
```

---

# 31. SVG — DEFER BY DEFAULT

Première question :

> Existe-t-il un consumer actuel nécessitant du vector UI runtime ?

Si non :

```text
SVG = DEFERRED CAPABILITY PRESSURE
```

Ne répondre alors qu'à :

```text
Could the current architecture prohibit it later?
Would correcting that prohibition later be expensive?
```

Sinon stop.

Ne pas faire une étude R&D SVG complète sans besoin.

---

# 32. WORLD-SPACE UI — DEFER BY DEFAULT

Pressure only :

```text
nameplate
health bar
interaction prompt
floating damage
```

Question unique par défaut :

> Le modèle interdit-il inutilement de rendre un subtree UI vers une autre surface/transform ?

Pas d'architecture world-space maintenant sans consumer.

---

# 33. INSPECTOR / HOT RELOAD / ADVANCED TOOLING — DEFER BY DEFAULT

Ne pas construire le kernel autour d'un inspector hypothétique.

Vérifier seulement si :

```text
stable debug IDs
tree/layout dumps
source locations
```

nécessitent une décision structurelle actuelle.

Sinon :

```text
DEFER
```

---

# 34. PERFORMANCE / DETERMINISM / MEMORY

Ne pas optimiser avant mesure.

Identifier les risques pertinents :

```text
tree rebuild
layout
allocations/frame
String churn
dynamic dispatch
cache invalidation
large lists
```

Définir les futures mesures :

```text
allocations/frame
layout time
render time
event routing time
memory
binary size delta
WASM size delta
compile impact
```

Sans valeurs inventées.

## Determinism

Identifier les sources pertinentes :

```text
HashMap ordering
floating rounding
time
input ordering
platform behavior
```

Définir seulement les garanties réalistes.

---

# 35. TESTING / HEADLESS — ARCHITECTURAL REQUIREMENT

Concevoir la stratégie de test avant le MFE.

Considérer :

## Unit

```text
constraints
state
focus
hit test
event routing
```

## Property-based

```text
deterministic layout
valid focus
bounds invariants
no invalid extents
```

## Golden layout

```text
description → exact rects
```

## Render snapshots

Seulement si valeur > fragilité.

## Input traces

```text
initial state
+ semantic inputs
→ actions/final state
```

## Replay

Si l'architecture s'y prête.

## Fuzzing

Seulement sur surfaces d'entrée réellement risquées.

## Headless requirement

Question obligatoire :

> Quelle proportion du système peut être validée sans fenêtre/GPU ?

Un bug UI corrigé devrait, autant que possible, devenir un test headless reproductible.

## Native/Web parity pressure

Sans transformer la Phase 0 en campagne de validation plateforme, identifier pour chaque mécanisme retenu :

```text
native assumption
Web/WASM assumption
known parity risk
future validation required
```

Tout mécanisme dépendant de :

```text
filesystem
threads
platform fonts
clipboard
pointer semantics
browser event ordering
asset loading
```

doit être explicitement signalé si pertinent.

La Phase 0 n'a pas à prouver la parité native/Web, mais elle ne doit pas l'oublier.

---

# 36. OBSERVABILITY — MINIMUM NECESSARY

Au minimum examiner la valeur de :

```text
state dump
focus dump
layout dump
event trace
```

Ne pas implémenter d'inspector.

Question :

> Un développeur ou agent peut-il diagnostiquer un bug UI important à partir d'un état textuel testable ?

---

# 37. ARCHITECTURE DECISION CONTRACT

Chaque décision importante doit utiliser :

```text
Problem
Evidence
Candidate
Alternatives
Trade-offs
Consumer pressure
Testability
Cost evidence
Failure mode
Revisit trigger
Decision confidence
```

## Decision confidence

Utiliser plusieurs dimensions :

```text
EVIDENCE-BACKED
CONSUMER-BACKED
PRIOR-ART-BACKED
HYPOTHESIS ONLY
REQUIRES MFE
UNKNOWN
```

Ne pas produire un seul `Confidence: HIGH/MEDIUM/LOW` global.

La provenance de `Evidence` doit respecter §12.2.

---

# 38. ADVERSARIES ACTING DURING DESIGN

Trois adversaires doivent agir **pendant chaque proposition** :

## Small Game

Peut éliminer une solution si elle impose trop de surface/coût/concepts.

## Rust Ergonomics

Peut éliminer une solution conceptuellement élégante mais manifestement pénible.

## Migration

Peut éliminer une solution incompatible avec une évolution behavior-constant réaliste.

Ils ne doivent pas attendre la revue finale.

---

# 39. ADVERSARIAL REVIEW FINAL

Après les architectures candidates, lancer au minimum :

## A — Browser Accident
## B — Enterprise Framework
## C — Performance
## D — Rust Ergonomics
## E — Small Game
## F — Highly Custom Game
## G — Test Engineer
## H — Migration
## I — Web/WASM
## J — Future Markup
## K — Accessibility/I18N
## L — API Evolution

Pour chaque finding :

```text
finding
severity
architecture affected
evidence
mitigation
residual risk
```

Ne pas inventer de findings pour remplir les adversaires : un adversaire peut conclure `NO MATERIAL FINDING` avec justification.

---

# 40. ARCHITECTURE COMPARISON

Comparer obligatoirement les options encore rationnellement pertinentes parmi :

```text
Architecture 0 — evolve existing src/ui
Architecture A — minimal subsystem
Architecture B — balanced subsystem
Architecture C — ambitious subsystem
```

Architecture C peut être éliminée tôt si les preuves ne la justifient pas.

Matrice :

| Criterion | 0 | A | B | C |
|---|---:|---:|---:|---:|
| Solves observed problems | | | | |
| Small-game cost | | | | |
| Migration risk | | | | |
| Conceptual Rust ergonomics | | | | |
| Testability | | | | |
| Pixel control | | | | |
| Multi-input | | | | |
| Extensibility | | | | |
| Runtime cost evidence | | | | |
| Dependency cost evidence | | | | |
| Future dead-end risk | | | | |
| Speculative machinery | | | | |

Ne pas favoriser l'architecture avec le plus de cases fonctionnelles.

Si une architecture est éliminée avant analyse détaillée, inscrire `ELIMINATED EARLY` et la raison au lieu d'inventer des évaluations.

---

# 41. ADVERSARIAL REDUCTION PASS

Après comparaison :

> Supprimer tout concept qui n'est pas nécessaire à la solution recommandée.

Produire :

```text
KEEP IN KERNEL
KEEP ABOVE KERNEL
OPTIONAL
DEFER
REJECT
UNKNOWN
```

Pour chaque concept important :

```text
UiTree
UiId
layout constraints
grid
theme
style classes
event propagation
feedback
audio
haptics
animation
markup
SVG
inspector
world-space
```

Le résultat final doit être **plus petit** que l'architecture initialement envisagée, sauf preuve contraire.

---

# 42. MFE SELECTION — RISK × UNCERTAINTY × INFORMATION GAIN

Le MFE n'est pas prédéfini.

Avant de le choisir produire une matrice :

| Candidate Experiment | Risk Addressed | Uncertainty | Cost | Information Gain |
|---|---|---|---|---|

Exemples possibles :

```text
stable identity + dynamic structure + focus restoration
minimal layout + two framebuffer sizes
semantic navigation
existing Ui migration adapter
frame transaction prototype
```

Les colonnes servent à comparer qualitativement les candidats.

Interdit :

```text
Risk = 4
Uncertainty = 5
Cost = 2
Score = 10
```

sans échelle explicitement définie et preuves justifiant chaque score.

Choisir le MFE qui réduit le plus l'incertitude critique au plus faible coût.

Le MFE peut être moins visuel qu'une mini-démo.

---

# 43. MFE_001_PROPOSAL

Le MFE retenu doit définir :

```text
hypotheses
why this experiment
scope
non-goals
candidate API
consumer
tests
measurements
native/Web validation pressure where relevant
human runtime gate
failure criteria
rollback boundary
STOP conditions
```

Le résultat futur devra pouvoir être :

```text
PASS
PASS WITH CONDITIONS
FAIL
```

Un FAIL doit permettre de réviser l'architecture.

---

# 44. MIGRATION MAP

Produire une migration incrémentale.

Pour chaque primitive existante :

```text
current symbol
classification
target concept
migration phase
compatibility strategy
deprecation timing
removal gate
```

Classification :

```text
KEEP
MOVE
GENERALIZE
WRAP
DEPRECATE LATER
DELETE LATER
UNRESOLVED
```

Processus :

```text
inventory
→ introduce proven primitive
→ compatibility/adaptation
→ migrate one real consumer
→ validate
→ migrate next
→ deprecate
→ remove only when unused
```

Éviter la coexistence indéfinie de deux systèmes.

---

# 45. LIVRABLES — COVERAGE, NOT FILE COUNT

Les thèmes suivants doivent être couverts.

La décomposition en fichiers n'est PAS obligatoire.

> **Prefer fewer substantial documents over many shallow documents.**

Destination :

```text
docs/ui/phase0/
```

Fichiers recommandés mais non imposés :

```text
README.md
EXISTING_UI_AUDIT.md
OBSERVED_PROBLEMS_AND_UNCERTAINTIES.md
PRIOR_ART.md
ARCHITECTURE_CANDIDATES.md
DEPENDENCY_DAG.md
FRAME_TRANSACTION_AND_EVENTS.md
MODULARITY_AND_COSTS.md
TEST_STRATEGY.md
MIGRATION_MAP.md
ADVERSARIAL_REVIEW.md
SYNTHESIS.md
MFE_001_PROPOSAL.md
```

Fusionner ou subdiviser si nécessaire.

Interdit :

```text
fichiers superficiels créés juste pour satisfaire une checklist
```

---

# 46. REQUIRED SYNTHESIS CONTENT

La synthèse doit contenir explicitement :

## Baseline

```text
repository
HEAD
branch
worktree
```

## Existing UI

```text
what already works
what is reusable
shortcomings assessment
```

## Observed problems

Priorisés, ou :

```text
NO STRUCTURAL PROBLEM FOUND
```

avec preuve.

## Highest-impact uncertainties

Jusqu'à 5, jamais remplies artificiellement.

## Research scope reduction

```text
required now
pressure only
defer
no need
```

## Consumer evidence provenance

Pour chaque consumer effectivement utilisé comme preuve :

```text
repo
commit/ref
files/symbols
local/remote
```

## Architectures

```text
0 / A / B / C
```

avec `ELIMINATED EARLY` autorisé.

## Dependency DAG

## Frame transaction model

## Event routing model

## Recommended direction

Possibilités :

```text
ARCHITECTURE 0
A
B
C
HYBRID
MORE RESEARCH
```

## Decision confidence per major decision

## Adversarial reduction

## Migration strategy

## MFE selected from risk matrix

## Unresolved risks

Jusqu'à 5 risques matériels, sans padding.

---

# 47. FAILURE-FIRST REVIEW

Pour la direction recommandée produire :

```text
up to 10 materially distinct ways this architecture could fail
```

Ne jamais compléter artificiellement jusqu'à 10.

Pour chacun :

```text
early signal
impact
mitigation
rollback boundary
```

Si seulement quelques failure modes sont matériels, s'arrêter là.

---

# 48. RUST API PSEUDO-USAGES

Seulement après réduction architecturale.

Montrer conceptuellement :

## Tiny game

```text
Panel + Text + Button
```

## Settings

```text
Tabs + Toggle + Slider
```

## Arcade

Seulement les concepts réellement retenus.

## Custom widget

Un exemple d'escape hatch.

Chaque pseudo-usage doit être étiqueté :

```text
CANDIDATE
NOT VALIDATED BY RUST COMPILATION
```

---

# 49. LEGACY COMPATIBILITY

Pour chaque primitive actuelle importante répondre :

```text
behavior-equivalent migration possible?
adapter possible?
temporary coexistence needed?
breaking change unavoidable?
removal gate?
```

Le futur système ne gagne pas le droit d'exister en cassant silencieusement les consumers.

---

# 50. PHASE 0 LIMITS ON MARKUP / SVG / ACCESSIBILITY / WORLD UI

La Phase 0 peut conclure :

```text
DEFER
```

sans rapport détaillé si aucune preuve ne justifie l'étude.

C'est un résultat positif.

Le prompt a explicitement le droit de **ne pas explorer** une capacité.

---

# 51. GIT DISCIPLINE

Créer une branche de documentation, par exemple :

```text
research/gpe-ui-phase0
```

Ne pas travailler directement sur `main`.

Modifier uniquement :

```text
docs/ui/phase0/**
```

Sauf justification documentaire exceptionnelle.

Ne modifier sous aucun prétexte :

```text
src/**
examples/**
Cargo.toml
Cargo.lock
assets/**
```

Commits suggérés :

```text
docs(ui): research GPE.UI phase 0
docs(ui): synthesize GPE.UI phase 0
```

Ne merge pas.

Push autorisé après validation.

---

# 52. VALIDATION AVANT COMMIT

Vérifier :

```bash
git diff --check
git status --short
```

Puis prouver :

```text
files outside docs/ui/phase0 modified: NONE
```

Aucune validation Rust n'est exigée si aucun code n'a changé.

---

# 53. TERMINAL STATUS

Deux états terminaux possibles seulement.

## Success

```text
COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW
```

## Blocked / insufficient evidence

```text
INCOMPLETE — BLOCKED
```

Ne jamais déclarer simultanément :

```text
BLOCKED
```

et :

```text
RESEARCH COMPLETE
```

---

# 54. FINAL REPORT TEMPLATE

Répondre avec une synthèse de ce type :

```text
GPE.UI PHASE 0 RESULT

Baseline:
<exact SHA>

Branch:
<name>

Gate 0 — Existing system understood:
PASS / FAIL

Shortcomings assessment:
<REAL SHORTCOMINGS IDENTIFIED / NO STRUCTURAL SHORTCOMING FOUND / INSUFFICIENT EVIDENCE>

Observed structural problems:
<material findings only, or NO STRUCTURAL PROBLEM FOUND>

Highest-impact uncertainties:
<up to 5; do not pad>

Research scope:
REQUIRED NOW:
- ...

PRESSURE ONLY:
- ...

DEFER:
- ...

NO CURRENT NEED:
- ...

Consumer evidence baselines:
- <repo @ commit/ref — local/remote — files/symbols>

Architecture 0:
<summary>

Architecture A:
<summary or ELIMINATED EARLY>

Architecture B:
<summary or ELIMINATED EARLY>

Architecture C:
<summary or ELIMINATED EARLY>

Recommended direction:
<0 / A / B / C / HYBRID / MORE RESEARCH>

Decision confidence:
- <decision>: <classification>
- <decision>: <classification>

Candidate dependency DAG:
<summary>

Frame transaction model:
<summary>

Event routing model:
<summary>

Small-game minimum:
<summary>

Current GPE UI primitives to preserve:
- ...

Legacy liabilities:
- ...

Concepts:
KEEP IN KERNEL:
- ...

KEEP ABOVE KERNEL:
- ...

OPTIONAL:
- ...

DEFER:
- ...

REJECT:
- ...

UNKNOWN:
- ...

Testing strategy:
<summary including native/Web parity pressure where relevant>

Markup:
DEFER / GO / MORE RESEARCH
Reason:
<short>

SVG:
DEFER / GO / MORE RESEARCH
Reason:
<short>

Migration strategy:
<summary>

MFE risk-selection:
<qualitative summary; no invented score>

MFE-001:
<summary>

Top unresolved risks:
<up to 5 material risks; do not pad>

Files created:
<list>

Files outside docs/ui/phase0 modified:
NONE

Commits:
<list>

Push:
<status>

PHASE 0 STATUS:
COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW
```

ou, si bloqué :

```text
PHASE 0 STATUS:
INCOMPLETE — BLOCKED

Blocking evidence gap:
<explanation>
```

---

# 55. FINAL STOP

Après les documents :

# STOP

Ne :

```text
code rien
crée aucune crate
ajoute aucune dependency
modifie aucun module UI existant
migre aucun consumer
implémente aucun widget
implémente aucun parser
implémente aucun SVG
implémente aucune Card Arcade
```

La prochaine étape autorisée est uniquement :

> **revue adversariale indépendante de la Phase 0 produite**

Aucun MFE ne peut commencer avant cette revue.

---

# 56. NORTH STAR MÉTHODOLOGIQUE

La Phase 0 n'a pas réussi si elle produit le plus gros document.

Elle a réussi si elle peut répondre avec des preuves à :

> **Quel est le minimum d'architecture UI dont GPE a réellement besoin maintenant ?**

et :

> **Quelles décisions devons-nous prendre maintenant pour éviter un dead-end, et lesquelles devons-nous explicitement refuser de prendre trop tôt ?**

Le résultat idéal peut parfaitement être :

```text
Architecture 0 wins.
Improve existing src/ui.
Do not create gpe-ui yet.
```

ou :

```text
A distinct subsystem is justified,
but only with a very small kernel.
```

ou :

```text
Evidence is insufficient.
Run a narrower research/probe before architecture.
```

ou même :

```text
No structural UI problem is currently demonstrated.
Do not create new architecture yet.
```

Toutes ces conclusions sont acceptables.

La seule conclusion interdite est celle qui prétend savoir plus que les preuves disponibles.
