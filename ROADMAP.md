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

## M0 — Spike plateforme

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

- `cargo run` produit une démonstration interactive reproductible ;
- aucun C/C++ ou binding vers olcPixelGameEngine n'est requis ;
- le chemin CPU framebuffer → GPU → écran est identifiable dans le code ;
- les responsabilités plateforme, rendu et démonstration ne sont pas inutilement entremêlées ;
- `cargo fmt --check`, `cargo clippy` et `cargo test` passent ;
- les versions Rust et dépendances principales sont documentées ;
- une mesure simple du temps de frame/FPS est disponible pour établir une baseline.

**Non-objectifs M0 :** sprites, audio, ECS, scène, UI, moteur de collision, asset manager, plugins, éditeur.

## M1 — Pixel Engine minimal

Transformer le spike en petite API utilisable :

- `Pixel` et couleurs ;
- `clear` ;
- `draw` / accès pixel borné ;
- lignes ;
- rectangles et rectangles pleins ;
- cercles et cercles pleins ;
- clavier/souris minimal ;
- timing ;
- exemple simple utilisant uniquement l'API publique.

Ajouter les premiers benchmarks des primitives CPU.

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

## M6 — Premier jeu réel

Construire un petit jeu complet avec le moteur. Ce milestone sert de test architectural : toute friction récurrente devient un candidat d'évolution du moteur.

Le jeu doit notamment tester :

- boucle complète ;
- inputs ;
- rendu ;
- assets si disponibles ;
- géométrie/collisions si nécessaires ;
- packaging minimal.

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

- WebAssembly ;
- gamepads ;
- resource packs ;
- tooling/éditeur ;
- réseau ;
- compute shaders ;
- ECS ou scene graph si un besoin concret les rend réellement pertinents.
