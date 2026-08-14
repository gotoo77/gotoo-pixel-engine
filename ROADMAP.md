# Roadmap — gotoo-pixel-engine

Cette roadmap est pilotée par des capacités observables. Chaque milestone doit
produire quelque chose d'exécutable, testable ou mesurable.

## Gouvernance

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce
> qu'elle pourrait être utile plus tard.

Corollaires :

- pas d'ECS par anticipation ;
- pas de scene graph par anticipation ;
- pas de framework UI par anticipation ;
- pas d'asset manager par anticipation ;
- pas d'optimisation sans mesure ;
- préférer une petite API explicite à une architecture générique prématurée.

## Phase historique — socle moteur ✅

### M0 — Spike plateforme

Acquis : fenêtre, framebuffer CPU, upload GPU, présentation `wgpu`, clavier,
delta-time, resize et premières mesures de performance.

### M1.1 — Pixel et framebuffer public

Acquis : `Pixel`, `Framebuffer`, couleurs de base, lecture/écriture de pixels et
tests de limites.

### M1.2 — Primitives de dessin CPU

Acquis : lignes, rectangles, rectangles pleins, cercles, cercles pleins,
clipping local et benchmarks.

### M1.3 — API de jeu et input

Acquis : `Game`, `Frame`, `GameResult`, clavier, souris, états
`pressed/held/released`, timing et reset input à la perte de focus.

### M1.5 — Portabilité WebAssembly

Acquis : build `wasm32-unknown-unknown`, WebGPU via `wgpu`, chemin de rendu
partagé et divergence Web confinée à la frontière plateforme.

## Validation par Snake ✅

Snake est le premier jeu réel ayant validé l'architecture du moteur.

Acquis :

- natif et Web/WASM ;
- rendu sRGB cohérent ;
- déploiement GitHub Pages ;
- clavier et tactile ;
- D-pad tactile privé au jeu ;
- texte bitmap, score, HUD et layout ;
- `Viewport`, letterbox/pillarbox et mapping input ;
- stockage local persistant ;
- audio one-shot natif/Web ;
- `ControlMap` avec clavier et gamepad ;
- menu natif `PLAY / CONTROLS / QUIT` réutilisant l'UI minimale commune.

Séparation validée :

```text
SnakeWorld
    métier pur du jeu

SnakeGame
    adaptation input, layout, HUD, replay, storage, audio

gotoo-pixel-engine
    plateforme, rendu, input, viewport, storage, audio
```

## Phase multi-jeux ✅ / 🚧

Le moteur n'est plus validé par un seul jeu. Les consommateurs actuels sont
Snake, Tetris, Space Invaders et Pong, avec Breakout en cours de validation.

### Tetris ✅

Tetris a validé un second gameplay, une grille différente et un second
consommateur de l'UI minimale.

Acquis :

- jeu jouable ;
- menu natif ;
- réutilisation des primitives UI communes ;
- validation de la philosophie « laisser les besoins spécifiques dans le jeu ».

### Space Invaders ✅

Space Invaders a poussé plus loin l'input et l'UI.

Acquis :

- gameplay jouable ;
- `ControlMap` ;
- menu principal et écran de contrôles ;
- diagnostic gamepad ;
- profils gamepad déclaratifs via `Game::gamepad_profile()` ;
- réglage du seuil numérique sans alourdir `Frame`.

### UI minimale partagée ✅

Les abstractions UI actuellement justifiées par plusieurs consommateurs sont :

- `draw_panel` ;
- `draw_text_centered` ;
- `draw_menu_item` ;
- `MenuState` ;
- `menu_up_pressed` ;
- `menu_down_pressed` ;
- `menu_confirm_pressed`.

Pas de composant générique, arbre UI, système de focus, callbacks ou thème tant
qu'un besoin concret supplémentaire ne les impose pas.

### Gamepad natif et profils ✅

Acquis :

- détection connexion/déconnexion ;
- boutons, D-pad et stick gauche normalisés ;
- gestion du D-pad centré observé sur le NEXT SNES Controller ;
- `GamepadProfile` et calibration d'axes ;
- probe visuel de diagnostic ;
- binding `ControlMap` acceptant n'importe quelle manette pour les jeux solo.

### M4.0 — Pong deux joueurs ✅

Pong a fourni le premier besoin réel d'affectation d'un périphérique à un
joueur précis.

Acquis :

- Pong V0 jouable à deux ;
- P1 clavier `W/S`, P2 clavier `Up/Down` ;
- première manette connectée -> P1 ;
- deuxième manette connectée -> P2 ;
- hotplug simple ;
- `ControlBinding::GamepadDevice(GamepadId, GamepadButton)` ;
- `ControlMap::bind_gamepad_device(...)` ;
- conservation du comportement « any gamepad » des jeux solo.

Aucun `PlayerManager` ou système générique de slots n'a été introduit : Pong ne
le justifie pas encore.

### M4.1 — Breakout / validation collisions 🚧

Objectif : ajouter un nouveau jeu consommateur qui exerce davantage les
collisions 2D et la destruction d'objets.

Validation attendue :

- raquette clavier + gamepad ;
- balle et rebonds murs/raquette ;
- grille de briques destructibles ;
- score et vies ;
- victoire / game over / restart ;
- menu partagé ;
- collisions gardées privées à Breakout pendant le premier passage.

Décision après validation : comparer les collisions de Pong et Breakout. Si le
même test AABB est réellement dupliqué et stable, promouvoir uniquement cette
petite primitive dans le moteur. Ne pas créer un système de physique général.

## État actuel du moteur

Le moteur possède maintenant :

- framebuffer CPU pixel-first ;
- primitives de dessin et texte bitmap ;
- clavier, souris, tactile et gamepad natif ;
- `ControlMap` avec bindings clavier, gamepad global et gamepad ciblé ;
- timing ;
- viewport et mapping surface -> framebuffer ;
- stockage local natif/Web ;
- audio one-shot natif/Web ;
- cible native ;
- cible WebAssembly/WebGPU ;
- UI immediate-mode minimale validée par plusieurs jeux ;
- plusieurs jeux consommateurs indépendants.

## Backlog futur non engagé

### Geometry2D

Candidat désormais concret, mais pas encore engagé. Pong et Breakout doivent
d'abord démontrer une duplication stable. Le premier candidat serait une
primitive AABB minimale, pas une bibliothèque de physique.

### Sprites et images

Besoin attendu : chargement d'image, représentation `Sprite`, dessin
complet/partiel, transparence et ownership Rust clair. À engager seulement avec
un jeu consommateur qui en a réellement besoin.

### Rendu GPU / decals

Explorer seulement lorsque des mesures ou un jeu réel montrent que le chemin
framebuffer CPU ne suffit plus.

### Gamepad Web

Le backend gamepad Web reste non engagé. Ne pas généraliser le backend natif
avant qu'un jeu Web demande réellement le support manette.

### Audio avancé

Le one-shot nécessaire aux jeux actuels existe. Mixer exposé, streaming ou
pipeline d'assets restent non engagés.

### Grimoire Javidx9

Ports sélectifs seulement, avec provenance claire, lorsqu'un besoin pédagogique
ou ludique concret apparaît.

## ECS

Pas d'ECS actuellement.

Snake, Tetris, Space Invaders, Pong et Breakout ne le justifient pas. Une
décision ECS ne serait défendable qu'à partir d'une duplication ou friction
observée sur plusieurs jeux.
