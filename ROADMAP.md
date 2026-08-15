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
- GPE Arcade, qui compose les cinq jeux dans un même runtime.

Cette phase a validé plusieurs abstractions qui possèdent maintenant plusieurs
consommateurs réels.

### `ControlMap` ✅

Acquis :

- actions logiques indépendantes des périphériques ;
- bindings clavier ;
- bindings « any gamepad » ;
- bindings gamepad ciblés pour Pong deux joueurs ;
- sources virtuelles tactiles via `VirtualPad` ;
- états `pressed`, `held`, `released` communs aux différentes sources.

### UI minimale partagée ✅

Abstractions actuellement justifiées :

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

Acquis :

- détection connexion/déconnexion ;
- boutons, D-pad et stick gauche normalisés ;
- backend natif `gilrs` ;
- backend Web basé sur la Gamepad API pour les mappings standards ;
- gestion du D-pad centré observé sur le NEXT SNES Controller ;
- `GamepadProfile` et calibration d'axes ;
- probe visuel de diagnostic ;
- binding gamepad global et binding ciblé par périphérique.

### M4.0 — Pong deux joueurs ✅

Pong a fourni le premier besoin réel d'affectation d'un périphérique à un
joueur précis sans introduire de `PlayerManager` généraliste.

### M4.1 — Breakout / collisions ✅

Breakout a validé :

- balle et rebonds ;
- raquette ;
- briques destructibles ;
- score, vies et niveaux ;
- game over / replay ;
- clavier, gamepad et tactile ;
- audio one-shot ;
- composition dans GPE Arcade.

Pong et Breakout utilisent tous deux des collisions AABB. La primitive
`Rect::intersects()` existe déjà dans le moteur : la consolidation consiste donc
à faire converger les consommateurs vers cette API existante plutôt qu'à créer
une nouvelle couche de physique.

## Phase actuelle — consolidation multi-jeux 🚧

Objectif : stabiliser ce que les consommateurs ont réellement démontré avant
d'ajouter de nouvelles capacités moteur.

### C1 — Garde-fous dépôt ✅

- CI native : format, tests, clippy warnings-as-errors, whitespace ;
- CI Web : compilation de tous les entrypoints WASM ;
- liste canonique des jeux Web partagée par les scripts ;
- déploiement Pages conservé comme étape distincte.

### C2 — Frontière uniforme des jeux ✅

Pong et Breakout séparent maintenant leur shell standalone de leur cœur
réutilisable :

```text
pong.rs              -> shell standalone
pong/game.rs         -> PongGame

breakout.rs          -> shell standalone
breakout/game.rs     -> BreakoutGame
```

Les entrypoints Web et GPE Arcade composent directement `PongGame` et
`BreakoutGame`. Le menu standalone ne fuit donc plus dans les contextes de
composition.

Aucun framework de scènes ou système de plugins n'a été introduit pour obtenir
cette frontière.

### C3 — Réutilisation des primitives existantes ✅

- Pong et Breakout utilisent `Rect::intersects()` au lieu de copies locales
  d'AABB ;
- Pause et Arcade réutilisent la même politique minimale de navigation menu ;
- aucune nouvelle couche de physique ou UI n'a été introduite.

### C4 — Politique de timing 🚧

Les jeux appliquent aujourd'hui des stratégies différentes face aux gros
`delta_time`. Définir un contrat moteur cohérent contre les pauses pathologiques
sans introduire un système de temps généraliste.

### C5 — Ownership de la calibration gamepad 🚧

`Game::gamepad_profile()` a été utile pour valider les profils, mais Arcade
montre que la calibration est liée au périphérique/runtime plus qu'au gameplay.
Étudier le déplacement minimal de cette responsabilité hors de `Game` avant
d'ajouter de nouvelles options de configuration.

### C6 — Dette locale Snake 🚧

- supprimer les chemins tactiles uniquement présents pour les tests lorsqu'ils
  dupliquent `VirtualPad` ;
- découper le fichier du jeu si cela améliore la cohésion ;
- conserver `SnakeWorld` indépendant du moteur.

### C7 — Documentation 🚧

Maintenir README, architecture et roadmap synchronisés avec les capacités
réellement présentes. Pour ce projet pédagogique, la documentation fait partie
du contrat d'architecture.

## État actuel du moteur

Le moteur possède maintenant :

- framebuffer CPU pixel-first ;
- primitives de dessin, texte bitmap et `Rect` ;
- clavier, souris, tactile et gamepad natif/Web ;
- `ControlMap` et `VirtualPad` ;
- timing par frame ;
- viewport et mapping surface -> framebuffer ;
- stockage local natif/Web ;
- audio one-shot natif/Web ;
- cible native ;
- cible WebAssembly/WebGPU ;
- UI immediate-mode minimale ;
- pause réutilisable ;
- plusieurs jeux consommateurs indépendants ;
- Arcade comme consommateur de composition.

## Backlog futur non engagé

### Geometry2D

Pas de bibliothèque Geometry2D générale actuellement. `Rect::intersects()`
couvre déjà le premier besoin AABB démontré. Ajouter seulement les opérations
suivantes lorsqu'un nouveau consommateur réel les impose.

### Sprites et images

Besoin attendu : chargement d'image, représentation `Sprite`, dessin
complet/partiel, transparence et ownership Rust clair. À engager avec un jeu
consommateur concret.

### Rendu GPU / decals

Explorer seulement lorsque des mesures ou un jeu réel montrent que le chemin
framebuffer CPU ne suffit plus.

### Audio avancé

Le one-shot nécessaire aux jeux actuels existe. Mixer exposé, streaming ou
pipeline d'assets restent non engagés.

### Grimoire Javidx9

Ports sélectifs seulement, avec provenance claire, lorsqu'un besoin pédagogique
ou ludique concret apparaît.

## ECS

Pas d'ECS actuellement.

Snake, Tetris, Space Invaders, Pong, Breakout et Arcade ne le justifient pas.
Une décision ECS ne serait défendable qu'à partir d'une duplication ou friction
observée sur plusieurs jeux plus complexes.
