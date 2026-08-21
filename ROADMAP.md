# Roadmap — gotoo-pixel-engine

Cette roadmap est pilotée par des capacités observables. Chaque milestone doit
produire quelque chose d'exécutable, testable ou mesurable.

## Gouvernance

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce
> qu'elle pourrait être utile plus tard.

Corollaires :

- pas d'ECS par anticipation ;
- pas de scene graph par anticipation ;
- pas de framework UI généraliste par anticipation ;
- pas d'asset manager par anticipation ;
- pas d'optimisation sans mesure ;
- préférer une petite API explicite à une architecture générique prématurée ;
- une abstraction déjà présente doit être réellement utilisée par ses
  consommateurs, sinon son placement ou son utilité doit être reconsidéré.

## Phase historique — socle moteur ✅

### M0 — Spike plateforme

Fenêtre, framebuffer CPU, upload GPU, présentation `wgpu`, clavier, delta-time,
resize et premières mesures de performance.

### M1.1 — Pixel et framebuffer public

`Pixel`, `Framebuffer`, couleurs de base, lecture/écriture de pixels et tests de
limites.

### M1.2 — Primitives de dessin CPU

Lignes, rectangles, rectangles pleins, cercles, cercles pleins, clipping local,
texte bitmap et benchmarks.

### M1.3 — API de jeu et input

`Game`, `Frame`, `GameResult`, clavier, souris, états `pressed/held/released`,
timing et reset input à la perte de focus.

### M1.5 — Portabilité WebAssembly

Build `wasm32-unknown-unknown`, WebGPU via `wgpu`, chemin de rendu partagé et
divergence Web confinée à la frontière plateforme.

## Validation par Snake ✅

Snake a été le premier jeu réel à valider l'architecture : gameplay pur,
présentation/adaptation et moteur restent séparés. Il a également poussé les
besoins de viewport, tactile, stockage local et audio one-shot.

## Phase multi-jeux ✅

Le moteur est désormais exercé par :

- Snake ;
- Tetris ;
- Space Invaders ;
- Pong deux joueurs ;
- Breakout ;
- Smart Boy Hero ;
- GPE Arcade, qui compose plusieurs jeux dans un même runtime.

Cette phase a validé plusieurs abstractions qui possèdent maintenant plusieurs
consommateurs réels.

### `ControlMap` ✅

- actions logiques indépendantes des périphériques ;
- bindings clavier ;
- bindings « any gamepad » ;
- bindings gamepad ciblés pour Pong deux joueurs ;
- sources virtuelles tactiles via `VirtualPad` ;
- états `pressed`, `held`, `released` communs aux différentes sources.

### UI minimale partagée ✅

- `draw_panel` ;
- `draw_text_centered` ;
- `draw_menu_item` ;
- `MenuState` ;
- aides de navigation menu ;
- `VirtualPad` ;
- `PauseGame`.

Pas de composant générique, arbre UI, callbacks, thème ou focus manager tant
qu'un besoin concret supplémentaire ne les impose pas.

### Gamepad natif et Web ✅

- détection connexion/déconnexion ;
- boutons, D-pad et stick gauche normalisés ;
- backend natif `gilrs` ;
- backend Web basé sur la Gamepad API ;
- `GamepadProfile` et calibration d'axes ;
- probe visuel de diagnostic ;
- binding gamepad global et binding ciblé par périphérique.

### Pong deux joueurs ✅

Premier besoin réel d'affectation d'un périphérique à un joueur précis sans
introduire de `PlayerManager` généraliste.

### Breakout / collisions ✅

Breakout et Pong utilisent `Rect::intersects()` pour leurs collisions AABB. La
consolidation consiste à faire converger les consommateurs vers cette primitive
existante plutôt qu'à créer une couche de physique.

## Phase actuelle — consolidation multi-jeux ✅

### C1 — Garde-fous dépôt ✅

- CI native : format, tests, clippy warnings-as-errors, whitespace ;
- CI Web : compilation de tous les entrypoints WASM ;
- liste canonique des jeux Web partagée par les scripts ;
- déploiement Pages conservé comme étape distincte.

### C2 — Frontière uniforme des jeux ✅

Les shells standalone restent séparés de leurs cœurs réutilisables lorsque la
composition l'exige. GPE Arcade consomme directement les implémentations de jeu,
sans framework de scènes ou système de plugins.

### C3 — Réutilisation des primitives existantes ✅

Les consommateurs convergent vers les primitives moteur déjà présentes avant
toute nouvelle abstraction de géométrie, UI ou input.

### C4 — Politique de timing ✅

`Frame::delta_time` représente du temps de simulation borné, pas une dette de
temps murale :

- le runtime plafonne un frame de simulation à 100 ms ;
- les transitions focus/resume réinitialisent la référence temporelle ;
- le temps brut reste utilisé pour le diagnostic du frame time et des FPS.

### C5 — Ownership de la calibration gamepad ✅

La calibration appartient au chemin runtime du périphérique, pas au gameplay.
`Frame::set_gamepad_profile` reste l'opération publique minimale nécessaire aux
écrans de configuration et outils de diagnostic.

### C6 — Dette locale Snake ✅

Le chemin tactile de Snake repose sur `VirtualPad` et la logique pure de grille,
serpent, nourriture et collisions reste isolée de l'adaptation runtime.

### C7 — Durcissement framebuffer ✅

`Framebuffer::draw_line` conserve la rasterisation visible de Bresenham sans
parcourir les portions gigantesques hors framebuffer. Les calculs intermédiaires
sont sûrs jusque sur des coordonnées extrêmes sans introduire une bibliothèque de
clipping générale.

## État actuel du moteur

Le moteur possède maintenant :

- framebuffer CPU pixel-first ;
- primitives de dessin, texte bitmap et `Rect` ;
- clavier, souris, tactile et gamepad natif/Web ;
- calibration gamepad par périphérique ;
- `ControlMap` et `VirtualPad` ;
- timing de simulation borné ;
- viewport et mapping surface -> framebuffer ;
- stockage local natif/Web ;
- audio one-shot et boucles identifiables natif/Web ;
- cibles native et WebAssembly/WebGPU ;
- UI immediate-mode minimale ;
- pause réutilisable ;
- plusieurs jeux consommateurs indépendants ;
- Arcade comme consommateur de composition.

## Backlog futur non engagé

### Geometry2D

Pas de bibliothèque Geometry2D générale actuellement. `Rect::intersects()`
couvre le premier besoin AABB démontré. Ajouter uniquement les opérations
suivantes lorsqu'un nouveau consommateur réel les impose.

### Sprites et images

Le premier chemin d'image partagé existe (`Image`, `Sprite`, blit et alpha).
Continuer à l'étendre seulement lorsqu'un consommateur réel démontre le besoin
d'atlas, d'animation ou d'un pipeline plus riche.

### Texte, fontes custom et i18n

Le framebuffer fournit aujourd'hui une fonte bitmap ASCII simple. Les extensions
UTF-8, Unicode, fontes custom, wrapping et catalogues de traduction restent des
capacités futures à valider par vertical slices réels, sans créer d'abord un
framework de localisation généraliste.

### Rendu GPU / decals

Explorer seulement lorsque des mesures ou un jeu réel montrent que le chemin
framebuffer CPU ne suffit plus.

### Audio avancé

One-shots et boucles identifiables existent. Streaming ou pipeline d'assets plus
riche restent non engagés tant qu'un jeu ne les exige pas.

### Assets runtime

Les assets embarqués restent le chemin simple par défaut. Une éventuelle
abstraction `AssetSource` devra être justifiée par plusieurs types d'assets et
plusieurs consommateurs avant d'entrer dans le moteur.

### Grimoire Javidx9

Ports sélectifs seulement, avec provenance claire, lorsqu'un besoin pédagogique
ou ludique concret apparaît.

## ECS

Pas d'ECS actuellement. Les jeux consommateurs existants et Arcade ne le
justifient pas. Une décision ECS ne serait défendable qu'à partir d'une duplication
ou friction observée sur plusieurs jeux plus complexes.

## Cible officielle future — GPE Android natif

Android est envisagé comme une cible GPE distincte du Web/WASM. L'objectif n'est
pas une WebView mais un moteur Rust natif produisant à terme un APK/AAB.

### A0 — Frontière plateforme Android

- distinguer explicitement Desktop / Web / Android dans les `cfg` nécessaires ;
- créer l'event loop Android avec `AndroidApp` sans changer le contrat `Game` ;
- conserver un backend gamepad Android minimal/no-op pour le premier slice ;
- obtenir une compilation `aarch64-linux-android` du moteur et de Snake.

### A1 — Snake Android vertical slice

```text
Snake GPE
   ↓
backend Android minimal
   ↓
APK ARM64
   ↓
installation sur smartphone réel
   ↓
rendu + input touch + son
```

Le même `SnakeGame` doit fonctionner avec un minimum de code spécifique au jeu
sur Windows, Linux, Web et Android.

### A2 — Lifecycle

- libérer/recréer les ressources de surface GPU lors des transitions
  suspend/resume ;
- réinitialiser timing et input lors des transitions de lifecycle ;
- valider reprise après verrouillage écran, changement d'application et retour.

### A3 — Services Android

- `LocalStorage` dans le répertoire applicatif Android ;
- orientation explicite ;
- fullscreen / system UI ;
- politique audio suspend/resume ;
- validation tactile sur appareil réel.

### A4 — Packaging Android

- chaîne NDK/Gradle minimale ;
- APK debug puis release ;
- signature ;
- AAB lorsque le vertical slice APK est stable ;
- CI Android seulement après validation locale sur appareil réel.

### A5 — Second consommateur

Porter ensuite Smart Boy Hero ou un autre jeu GPE plus complexe. Le second
consommateur doit démontrer que le backend Android appartient au moteur et non à
Snake.
