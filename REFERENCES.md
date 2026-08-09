# Références et provenance

Ce document recense les projets utilisés comme références conceptuelles, pédagogiques ou techniques.

L'objectif est de maintenir une distinction explicite entre :

- **référence** : lecture/comparaison d'un projet ;
- **réimplémentation** : reconstruction d'une idée ou d'un comportement ;
- **port** : adaptation identifiable de code ou d'un algorithme ;
- **code repris** : code source dérivé directement, qui doit conserver les obligations de licence et d'attribution applicables.

## OneLoneCoder/olcPixelGameEngine

https://github.com/OneLoneCoder/olcPixelGameEngine

Référence canonique pour la philosophie PixelGameEngine et son API C++.

Points d'intérêt :

- boucle applicative très simple ;
- framebuffer/pixel drawing ;
- sprites ;
- layers ;
- decals et rendu GPU ;
- input ;
- extensions.

Licence : OLC-3. Toute dérivation directe doit respecter les conditions de cette licence.

## OneLoneCoder/Javidx9

https://github.com/OneLoneCoder/Javidx9

Archive majeure de code accompagnant les vidéos et expérimentations de Javidx9.

Utilisation envisagée : source pédagogique et algorithmique pour des ports Rust sélectionnés : tilemaps, pathfinding, génération procédurale, raycasting, rendu logiciel, networking et prototypes de jeux.

Chaque port identifiable devra mentionner précisément sa provenance.

## OneLoneCoder/olcUTIL_Geometry2D

https://github.com/OneLoneCoder/olcUTIL_Geometry2D

Référence pour une future crate de géométrie 2D indépendante : points, lignes, cercles, rectangles, triangles, rays et opérations telles que `contains`, `overlaps`, `intersects`, `project`, `collide`, `reflect` et `closest`.

Objectif : concevoir une API Rust idiomatique plutôt que traduire mécaniquement le header C++.

## OneLoneCoder/olcSoundWaveEngine

https://github.com/OneLoneCoder/olcSoundWaveEngine

Référence conceptuelle pour l'audio et la synthèse. Aucun port n'est décidé : l'écosystème Rust existant sera évalué avant toute implémentation.

## OneLoneCoder/olcEditor

https://github.com/OneLoneCoder/olcEditor

Référence éventuelle pour l'outillage futur. Hors périmètre initial.

## sadikovi/olcPixelGameEngine-rs

https://github.com/sadikovi/olcPixelGameEngine-rs

Binding Rust vers olcPixelGameEngine C++.

Intérêt :

- correspondance API Rust/C++ ;
- exemples de traduction des conventions OLC ;
- possibilité de comparer facilement des exemples avec les vidéos originales.

Ce projet n'est pas retenu comme socle : notre objectif initial est un runtime Rust sans dépendance C++ obligatoire.

## GreenDog72/olc-pge

https://github.com/GreenDog72/olc-pge

Réimplémentation Rust non officielle d'olcPixelGameEngine.

Intérêt : adaptation de concepts C++ aux contraintes d'ownership et de mutabilité de Rust. Référence architecturale uniquement pour l'instant.

## Maix0/pixel_engine

https://github.com/Maix0/pixel_engine

Recréation Rust du concept PixelGameEngine reposant notamment sur `wgpu`/`winit`.

C'est une référence technique importante pour le bootstrap de `gotoo-pixel-engine`, mais le projet n'est volontairement pas forké : nous souhaitons construire un dépôt autonome, choisir une stack Rust actuelle et conserver la liberté de conception.

## Politique de provenance

Pour toute contribution inspirée d'une source externe :

1. identifier la source ;
2. vérifier sa licence avant de reprendre du code ;
3. privilégier la compréhension puis la réimplémentation lorsqu'elle est appropriée ;
4. conserver les notices requises pour tout code dérivé ;
5. documenter les ports significatifs dans ce fichier ou à proximité du code concerné.

Ce document n'est pas une analyse juridique des licences ; il sert de registre technique de provenance.
