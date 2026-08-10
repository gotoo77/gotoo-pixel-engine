# Roadmap — gotoo-pixel-engine

Cette roadmap est pilotee par des capacites observables. Chaque milestone doit
produire quelque chose d'executable, testable ou mesurable.

## Gouvernance

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce
> qu'elle pourrait etre utile plus tard.

Corollaires :

- pas d'ECS par anticipation ;
- pas de scene graph par anticipation ;
- pas de framework UI par anticipation ;
- pas d'asset manager par anticipation ;
- pas d'optimisation sans mesure ;
- preferer une petite API explicite a une architecture generique prematuree.

## Phase Historique

### M0 — Spike plateforme ✅

**Statut : termine.** Valide avec le commit `38fd55f`
(`Implement M0 platform spike`).

But : valider le socle Rust + `winit` + `wgpu`.

Acquis :

- ouverture de fenetre ;
- framebuffer CPU ;
- upload du framebuffer vers une texture GPU ;
- presentation via `wgpu` ;
- fermeture et input clavier minimal ;
- delta-time ;
- gestion du resize sans crash ;
- premiere baseline FPS/frame time.

### M1.1 — Pixel et framebuffer public ✅

**Statut : termine.** Valide avec le commit `f79d512`
(`Add public pixel framebuffer API`).

Acquis :

- `Pixel` RGBA ;
- couleurs constantes de base ;
- `Framebuffer::new`, `clear`, `draw`, `pixel`, `as_rgba8` ;
- tests unitaires sur les couleurs, l'ecriture et les limites.

### M1.2 — Primitives de dessin CPU ✅

**Statut : termine.** Valide avec le commit `5524ec5`
(`Add CPU drawing primitives`).

Acquis :

- lignes ;
- rectangles et rectangles pleins ;
- cercles et cercles pleins ;
- clipping local au framebuffer ;
- benchmarks CPU reproductibles.

### M1.3 — Input, timing et exemple public ✅

**Statut : termine.** Valide avec le commit `54afd0c`
(`Implement M1.3 input timing API`).

Acquis :

- trait `Game` ;
- `Frame` ;
- `GameResult` ;
- clavier et souris ;
- etats `pressed`, `held`, `released` ;
- `delta_time` ;
- sortie controlee par le jeu ;
- validation de configuration ;
- reset input a la perte de focus.

### M1.5 — Portabilite WebAssembly ✅

**Statut : termine.** Valide avec le commit `9a2cb20`
(`Add WebAssembly browser support`).

Acquis :

- build `wasm32-unknown-unknown` ;
- surface WebGPU via le meme chemin `wgpu` ;
- exemple Web partageant le code metier de demo ;
- divergence Web confinee a la frontiere plateforme et au point d'entree WASM.

## Phase De Validation Par Snake

Snake a remplace le role anciennement prevu pour un lointain M6 : il est devenu
le premier jeu reel de validation directement dans la serie M1.x. Les anciennes
intentions sont conservees, mais l'histoire reelle est celle-ci.

### Snake natif et Web ✅

- `485506a` — `Add native Snake example`
- `b56f957` — `Add Web Snake and preserve sRGB rendering`
- `cca3ba6` — `Deploy Web Snake with GitHub Pages`

Acquis :

- Snake comme consommateur normal de l'API moteur ;
- version native ;
- version Web/WASM ;
- preservation du rendu sRGB ;
- workflow GitHub Pages.

### Input tactile et controles Snake ✅

- `dfbc075` — `Add touch input and Snake swipe controls`
- `da4f29e` — `Replace Snake swipe with touch D-pad`

Acquis :

- `TouchPhase`, `Touch`, `Input::touches()` ;
- mapping tactile vers coordonnees framebuffer ;
- convergence clavier/tactile vers les directions metier Snake ;
- D-pad tactile prive a Snake.

### Texte bitmap, score, HUD et layout Snake ✅

- `8a478ea` — `Add Snake score and bitmap text UI`
- `cce1e13` — `Separate Snake playfield HUD and touch controls`
- `a93a790` — `Add Snake interaction layout modes`

Acquis :

- texte bitmap dans `Framebuffer` ;
- score courant ;
- HUD ;
- separation playfield / HUD / controles ;
- layout clavier sans zone tactile ;
- layout tactile avec D-pad.

### Viewport moteur ✅

- `4039478` — `Add viewport-aware rendering and input`

Acquis :

- `Size`, `Rect`, `Viewport` publics ;
- conservation du ratio framebuffer ;
- letterbox/pillarbox ;
- rendu et input utilisant la meme transformation ;
- input hors viewport -> `None` ;
- resize/rotation sans recreation du monde de jeu.

### Stockage local persistant ✅

- `7fe947d` — `Add persistent Snake high score`

Acquis :

- trait `LocalStorage` ;
- backend natif fichier local via `directories` ;
- backend Web via `localStorage` ;
- `NoopStorage` pour tests ;
- BEST Snake persistant ;
- erreurs de stockage non bloquantes.

### Audio one-shot natif/Web ✅

- `c60602a` — `Add cross-platform audio support to Snake`

Acquis :

- trait `Audio` ;
- `SoundId` ;
- decode WAV PCM 16-bit mono/stereo 44100/48000 Hz ;
- backend natif via `rodio`/`cpal` ;
- backend Web via WebAudio ;
- `NoopAudio` pour tests ;
- sons Snake `snake.eat` et `snake.death` ;
- erreurs audio non bloquantes.

## Etat Actuel

Le moteur possede maintenant :

- framebuffer CPU pixel-first ;
- primitives de dessin et texte bitmap ;
- clavier, souris, tactile ;
- timing ;
- viewport et mapping surface -> framebuffer ;
- stockage local natif/Web ;
- audio one-shot natif/Web ;
- cible native ;
- cible WebAssembly/WebGPU ;
- premier jeu reel complet : Snake.

Snake est considere comme validation architecturale de la separation :

```text
SnakeWorld
    metier pur du jeu

SnakeGame
    adaptation input, layout, HUD, replay, storage, audio

gotoo-pixel-engine
    plateforme, rendu, input, viewport, storage, audio
```

## Prochaine Phase : Multi-Jeux

### Candidat suivant : Tetris

Tetris est le candidat naturel pour valider le moteur avec un deuxieme jeu sans
generaliser trop tot.

Il peut reutiliser immediatement :

- framebuffer et primitives ;
- texte bitmap ;
- input ;
- timing ;
- viewport ;
- storage pour un score local ;
- audio one-shot ;
- cible Web.

Il doit surtout reveler les prochains besoins reels du moteur : grille
differente, pieces, rotation, gravity, lock delay eventuel, preview, lignes,
pause ou etats de jeu. Ces besoins doivent d'abord rester prives a Tetris tant
qu'une duplication entre plusieurs jeux n'est pas observee.

## Backlog Futur Non Engage

### Sprites et images

Toujours futur. Besoin attendu : chargement d'image, representation `Sprite`,
dessin complet/partiel, transparence et ownership Rust clair.

### Rendu GPU / decals

Toujours futur. Explorer seulement lorsque des mesures ou un jeu reel montrent
que le chemin framebuffer CPU ne suffit plus.

### Geometry2D

Toujours futur. Possible crate independante inspiree de `olcUTIL_Geometry2D`,
mais aucun besoin actuel ne la force.

### Grimoire Javidx9

Toujours futur. Ports selectifs seulement, avec provenance claire, lorsque le
projet en a l'usage pedagogique ou ludique.

### Audio

L'audio minimal necessaire a Snake est realise. Un systeme audio plus large,
un mixer expose, du streaming ou un asset pipeline restent non engages.

### Snake

Le premier Snake complet est realise. Les evolutions futures de Snake ne doivent
pas etre confondues avec des besoins moteur generiques.

## ECS

Pas d'ECS actuellement.

Snake ne le justifie pas. Tetris ne le justifie pas a priori. Space Invaders ne
le justifiera pas automatiquement. Une decision ECS ne serait defendable qu'a
partir d'une duplication ou friction observee sur plusieurs jeux.
