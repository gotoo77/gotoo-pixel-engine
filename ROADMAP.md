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

## Phase actuelle — consolidation multi-jeux ✅

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

### C4 — Politique de timing ✅

`Frame::delta_time` représente maintenant du temps de simulation borné, pas une
dette de temps murale :

- le runtime plafonne un frame de simulation à 100 ms ;
- les transitions focus/resume réinitialisent la référence temporelle ;
- le temps brut reste utilisé pour le diagnostic du frame time et des FPS ;
- Pong et Breakout consomment le contrat commun au lieu d'appliquer leur propre
  clamp ;
- leurs substeps restent une stratégie de collision, pas une politique de
  timing.

Une suspension navigateur ou un stall long ne peut donc plus réinjecter plusieurs
secondes de simulation d'un coup. Aucun `TimeSystem` ni paramétrage générique n'a
été ajouté.

### C5 — Ownership de la calibration gamepad ✅

La calibration appartient maintenant au chemin runtime du périphérique, pas au
gameplay :

- `Game::gamepad_profile()` a été supprimé ;
- l'état de profil est conservé par périphérique dans le sous-système `Input` ;
- les backends natif et Web lisent ce même état avant de normaliser les entrées ;
- `Frame` expose uniquement `set_gamepad_profile`, l'opération publique requise
  par les écrans de configuration réellement présents ;
- le probe et le menu standalone de Space Invaders utilisent ce setter ;
- Pause et Arcade n'ont plus à relayer une responsabilité de périphérique ;
- le profil d'un périphérique est supprimé à sa déconnexion.

La structure publique de `Frame` n'a gagné aucun champ obligatoire et les jeux
ordinaires continuent simplement à consommer un `Input` déjà normalisé. Aucun
`DeviceManager`, registre de configuration ou système générique de périphériques
n'a été introduit.

### C6 — Dette locale Snake ✅

Le chemin tactile de Snake repose désormais sur une seule implémentation réelle :
`VirtualPad`.

- les anciens `TouchControls`, `DPadTracker`, contacts et zones compilés seulement
  pour les tests ont été supprimés ;
- les transitions tactiles, déplacements entre zones, multi-contact et reset
  sont testés directement dans `VirtualPad` ;
- les tests Snake vérifient uniquement le câblage de ses quatre actions vers le
  D-pad, le replay et les règles propres au jeu ;
- la logique pure de grille, serpent, nourriture, collisions et file de virages
  est isolée dans `snake/world.rs`, sans dépendance au moteur ;
- `snake/game.rs` conserve l'adaptation input, timing, layout, rendu, stockage et
  audio.

```text
snake/game.rs   -> adaptation runtime / présentation
snake/world.rs  -> modèle et règles de jeu purs
```

Aucun framework de scènes, système de monde ou abstraction moteur supplémentaire
n'a été introduit pour ce découpage.

### C7 — Durcissement final et documentation ✅

`Framebuffer::draw_line` conserve la rasterisation visible de Bresenham sans
parcourir les portions gigantesques situées hors du framebuffer :

- seules les étapes de l'axe majeur susceptibles d'être visibles sont parcourues ;
- le nombre d'itérations est donc borné par la largeur ou la hauteur du
  framebuffer, pas par l'écart entre les coordonnées du segment ;
- la phase de rasterisation d'origine est préservée aux frontières au lieu de
  relancer Bresenham depuis des extrémités arrondies ;
- les spans horizontaux, verticaux et diagonaux allant de `i32::MIN` à
  `i32::MAX` sont couverts par les tests ;
- les calculs intermédiaires utilisent `i128` lorsque nécessaire pour éviter les
  débordements.

Aucune bibliothèque Geometry2D ni abstraction générique de clipping n'a été
introduite. La roadmap et l'architecture documentent l'état final de cette passe ;
le README reste inchangé sur ce point purement interne, son contrat public ne
changeant pas.

## État actuel du moteur

Le moteur possède maintenant :

- framebuffer CPU pixel-first ;
- primitives de dessin, texte bitmap et `Rect`, avec coût de rasterisation des
  lignes borné à leur portion potentiellement visible ;
- clavier, souris, tactile et gamepad natif/Web ;
- calibration gamepad par périphérique possédée par le runtime d'input ;
- `ControlMap` et `VirtualPad` ;
- timing de simulation borné par frame ;
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

### Texte, fontes custom et i18n

Void Canticle fournit maintenant un besoin concret : interface et narration
destinées à être traduites dans plusieurs langues, dont le japonais, avec une
direction artistique typographique propre au jeu plutôt que la seule fonte ASCII
5x7 intégrée au framebuffer.

Capacités à valider par slices, sans créer d'abord un framework de localisation
généraliste :

- chaîne texte UTF-8 de bout en bout ;
- fonte asset configurable, avec possibilité de fontes bitmap/pixel custom ;
- mapping Unicode -> glyphes au lieu de supposer un alphabet ASCII fixe ;
- stratégie de glyphes japonais/CJK : atlas adapté ou génération/subsetting à
  partir du corpus réel de traductions, afin de ne pas embarquer aveuglément un
  atlas gigantesque ;
- métriques de glyphes, alignement, largeur variable et mesure de texte ;
- retour à la ligne/wrapping et layouts qui tolèrent des traductions plus longues ;
- fallback de fonte/glyphe visible et diagnostic des caractères absents ;
- séparation entre les identifiants stables de texte et leur traduction ;
- catalogues de locale (premiers objectifs : anglais, français, japonais),
  interpolation de valeurs, locale de fallback et diagnostic des clés manquantes ;
- possibilité pour un jeu de choisir plusieurs rôles typographiques (HUD,
  dialogue, titre) sans imposer ces choix artistiques au moteur.

Le premier vertical slice devra prouver un même écran Void Canticle rendu en
anglais, français et japonais avec une fonte custom, en natif et sur Web. Le
catalogue i18n peut rester local à Void Canticle tant qu'un second jeu ne justifie
pas une petite abstraction partagée dans GPE.

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

## Phase distribution native — P1 ✅

Void Canticle est le premier consommateur de distribution native GPE :

- `scripts/dev.py package-native void-canticle` effectue un build `--release` ;
- Windows x86_64 est construit sur `windows-latest` ;
- Linux x86_64 est construit sur `ubuntu-latest` ;
- les archives contiennent le binaire public, la licence et les assets runtime
  explicitement requis par le jeu ;
- les archives sont publiées comme artefacts GitHub Actions ;
- un second job, sans checkout du dépôt ni installation de Rust, télécharge,
  extrait et lance le livrable pour tester le package lui-même ;
- le smoke Linux vérifie également les dépendances dynamiques avec `ldd` ;
- un tag `void-canticle-v*` crée ou met à jour la GitHub Release correspondante
  après succès des deux plateformes.

Le premier slice reste volontairement descriptif par jeu dans `scripts/dev.py`.
Pas de `xtask`, d'asset manager ou de framework de distribution tant qu'un second
jeu ne démontre pas un besoin supplémentaire.

## Cible officielle future — GPE Android natif

Android devient une cible GPE distincte du Web/WASM. L'objectif n'est pas une
WebView ou Chrome mais un moteur Rust natif produisant à terme un APK/AAB.

La stack actuellement versionnée est déjà proche de cette cible mais la frontière
« natif = tout ce qui n'est pas WASM » doit être affinée :

- `winit 0.30.13` fournit le chemin Android basé sur `AndroidApp` /
  `android_main` et impose de traiter correctement le lifecycle ;
- `wgpu 30.0.0` fournit le rendu Android, avec Vulkan comme premier backend à
  valider ;
- `rodio 0.22.2` via `cpal 0.17.3` est un candidat crédible pour l'audio Android
  et doit être testé avant d'introduire un backend audio différent ;
- `gilrs 0.11.2` n'est pas un backend Android générique : le premier slice
  Android utilisera le tactile et pourra fournir un backend gamepad no-op ;
- le stockage desktop basé sur `directories` ne doit pas être supposé correct
  sur Android ; un backend `LocalStorage` Android devra utiliser les chemins
  applicatifs fournis par la plateforme ;
- `Game`, `Frame`, `Framebuffer`, `ControlMap` et `VirtualPad` doivent rester le
  contrat commun consommé par les jeux.

La topologie cible devient :

```text
                    GPE Game
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       Desktop        Web        Android
      Win/Linux       WASM        Native
```

### A0 — Frontière plateforme Android

- distinguer explicitement Desktop / Web / Android dans les `cfg` nécessaires ;
- créer l'event loop Android avec `AndroidApp` sans changer le contrat `Game` ;
- conserver un backend gamepad Android minimal/no-op pour le premier slice ;
- obtenir une compilation `aarch64-linux-android` du moteur et de Snake.

### A1 — Snake Android vertical slice

Critère de réussite :

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

Porter ensuite SBH ou Void Canticle sans fork majeur du gameplay. Le second
consommateur doit démontrer que le backend Android appartient à GPE et non à
Snake. Void Canticle est un candidat naturel pour valider ensuite le portrait
vertical sur smartphone.