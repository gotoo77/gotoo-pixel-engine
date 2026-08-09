# Roadmap — gotoo-pixel-engine

Cette roadmap est volontairement pilotée par des capacités observables. Chaque milestone doit produire quelque chose d'exécutable, testable ou mesurable.

## Règle de gouvernance

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce qu'elle pourrait être utile un jour.

Corollaires :

- pas d'ECS par anticipation ;
- pas de scene graph par anticipation ;
- pas de système de plugins par anticipation ;
- pas d'éditeur avant qu'un jeu démontre son besoin ;
- pas d'optimisation sans mesure ;
- préférer une petite API explicite à une architecture générique prématurée.

## M0 — Spike plateforme ✅

**Statut : terminé.** Validé avec le commit `38fd55f` (`Implement M0 platform spike`) et un test interactif humain de la fenêtre et du framebuffer.

**But :** valider le socle technique Rust + `winit` + `wgpu`.

Livrable minimal :

1. ouvrir une fenêtre ;
2. créer un framebuffer CPU possédé par le programme ;
3. modifier ses pixels côté CPU ;
4. transférer le framebuffer vers une texture GPU ;
5. afficher cette texture dans la fenêtre ;
6. gérer au minimum fermeture + une entrée clavier ;
7. calculer un delta-time exploitable ;
8. redimensionner ou gérer explicitement le redimensionnement sans crash ;
9. disposer d'une commande documentée pour lancer le spike.

### Critères de sortie M0

- [x] `cargo run` produit une démonstration interactive reproductible ;
- [x] aucun C/C++ ou binding vers olcPixelGameEngine n'est requis ;
- [x] le chemin CPU framebuffer → GPU → écran est identifiable dans le code ;
- [x] les responsabilités plateforme, rendu et démonstration ne sont pas inutilement entremêlées ;
- [x] `cargo fmt --check`, `cargo clippy` et `cargo test` passent ;
- [x] les versions Rust et dépendances principales sont documentées ;
- [x] une mesure simple du temps de frame/FPS est disponible pour établir une baseline.

**Non-objectifs M0 :** sprites, audio, ECS, scène, UI, moteur de collision, asset manager, plugins, éditeur.

## M1 — Pixel Engine minimal

Transformer le spike en première petite API de moteur réellement utilisable. M1 est volontairement découpé en étapes bornées : une étape doit être validée avant de passer à la suivante.

### M1.1 — Pixel et framebuffer public ✅

**Statut : terminé.** Validé avec le commit `f79d512` (`Add public pixel framebuffer API`).

**But :** définir la primitive fondamentale du moteur et une API CPU sûre et testable.

- [x] type `Pixel` RGBA explicite ;
- [x] constantes/couleurs de base utiles ;
- [x] `clear(Pixel)` ;
- [x] `draw(x, y, Pixel)` avec comportement borné défini ;
- [x] conserver un chemin simple pour les écritures dont les coordonnées sont déjà garanties ;
- [x] adapter la démo pour utiliser cette API lorsque pertinent ;
- [x] tests unitaires des couleurs, de `clear` et des limites de `draw`.

**Non-objectifs M1.1 :** lignes, rectangles, cercles, sprites, API GPU publique, ECS ou refonte générale de l'architecture.

### M1.2 — Primitives de dessin CPU

**But :** obtenir les primitives pixel-first nécessaires aux premiers prototypes.

- lignes ;
- rectangles ;
- rectangles pleins ;
- cercles ;
- cercles pleins ;
- clipping/comportement hors framebuffer explicitement défini ;
- tests sur les cas nominaux, limites et formes partiellement hors écran ;
- premiers benchmarks reproductibles des primitives CPU.

Les algorithmes doivent rester lisibles avant d'être micro-optimisés.

### M1.3 — Input, timing et exemple public

**But :** permettre à un petit jeu de dépendre de l'API du moteur plutôt que de manipuler directement `winit` ou les détails du spike.

- clavier minimal ;
- souris minimale ;
- delta-time accessible au jeu ;
- distinction utile entre état maintenu et transitions (`pressed`/`released`) si elle reste simple ;
- exemple interactif utilisant uniquement l'API publique prévue pour un jeu ;
- documentation de la boucle minimale d'une application.

### Critères de sortie M1

- l'exemple ne dépend pas directement des détails internes du renderer ;
- les primitives publiques nécessaires au milestone sont documentées et testées ;
- les entrées et le timing sont exploitables depuis le code de jeu ;
- les benchmarks CPU donnent une première baseline reproductible ;
- `cargo fmt --check`, `cargo clippy -- -D warnings` et `cargo test` passent ;
- aucune abstraction majeure non justifiée par M1 n'a été introduite.

## M1.5 — Spike de portabilité WebAssembly

**But :** vérifier tôt que l'architecture du moteur reste compatible avec un navigateur et qu'un même code de démonstration peut cibler desktop et WebAssembly.

Ce milestone est un spike de portabilité, pas la construction d'une plateforme web complète.

- compiler le moteur pour une cible WebAssembly adaptée au navigateur ;
- ouvrir une surface de rendu web et afficher le framebuffer via le chemin `wgpu`/web disponible ;
- conserver le même code métier de démonstration entre desktop et navigateur autant que raisonnablement possible ;
- documenter la commande de build et de lancement local ;
- identifier explicitement les divergences plateforme nécessaires ;
- vérifier qu'aucune dépendance introduite jusqu'à M1 ne bloque inutilement la cible web.

### Critères de sortie M1.5

- une démonstration s'exécute nativement et dans un navigateur à partir du même dépôt ;
- le code métier de la démonstration n'est pas dupliqué par plateforme ;
- les adaptations web restent confinées à la frontière plateforme ;
- le chemin framebuffer CPU → GPU → écran reste fonctionnel ;
- les limitations connues sont documentées.

**Non-objectifs M1.5 :** audio web, asset pipeline web complet, hébergement public, PWA, réseau, optimisation spécifique navigateur.

## M2 — Sprites et images

- chargement d'image ;
- représentation `Sprite` ;
- dessin complet et partiel ;
- transparence ;
- transformations nécessaires aux premiers jeux ;
- stratégie d'ownership Rust claire.

## M3 — Rendu GPU / decals

Explorer un chemin GPU complémentaire au framebuffer CPU :

- textures GPU ;
- decals/quads ;
- batching lorsque justifié par les mesures ;
- transformations ;
- transparence/blending ;
- benchmarks CPU vs GPU sur des scénarios documentés.

## M4 — Geometry2D

Créer une crate géométrique Rust indépendante, inspirée de `olcUTIL_Geometry2D` :

- points, segments, rectangles, cercles, triangles, rays ;
- `contains` ;
- `overlaps` ;
- `intersects` ;
- puis projection/collision/réflexion selon les besoins réels.

L'API doit être pensée en Rust plutôt que traduite mécaniquement du C++.

## M5 — Audio

Déterminer les besoins à partir d'un jeu réel puis sélectionner une brique Rust existante ou implémenter la couche minimale nécessaire. Ne pas porter `olcSoundWaveEngine` par principe.

## M6 — Premier jeu réel : Snake

Construire un Snake petit mais complet avec le moteur. Ce milestone sert de test d'acceptation architectural : le jeu doit pouvoir être écrit comme consommateur normal de `gotoo-pixel-engine`, sans contourner l'API ni manipuler les entrailles du renderer.

Le jeu doit notamment tester :

- boucle complète ;
- inputs ;
- rendu pixel-first ;
- déplacement à cadence contrôlée ;
- nourriture et croissance ;
- collisions avec le corps et les limites ;
- score et redémarrage minimal ;
- packaging/exécution minimal ;
- cible navigateur également lorsque M1.5 et les dépendances introduites depuis restent compatibles.

Toute friction récurrente rencontrée pendant Snake devient un candidat d'évolution du moteur, pas une raison de contourner son API.

## M7 — Grimoire Javidx9

Porter sélectivement des expériences et algorithmes de `OneLoneCoder/Javidx9`, en privilégiant la compréhension et une réécriture Rust idiomatique :

- tilemaps ;
- pathfinding ;
- génération procédurale ;
- raycasting ;
- rendu logiciel 3D ;
- autres expériences selon leur intérêt.

Chaque port doit indiquer clairement sa provenance.

## Plus tard, seulement si justifié

- gamepads ;
- resource packs ;
- tooling/éditeur ;
- réseau ;
- compute shaders ;
- ECS ou scene graph si un besoin concret les rend réellement pertinents.
