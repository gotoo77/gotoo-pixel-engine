# Architecture — gotoo-pixel-engine

## Philosophie

Le moteur doit rester suffisamment petit pour qu'un développeur puisse comprendre le chemin complet entre son code de jeu et le pixel affiché.

L'architecture initiale vise donc ceci :

```text
jeu
 │
 ▼
API gotoo-pixel-engine
 │
 ├── état / timing / input
 │
 ├── framebuffer CPU
 │       │
 │       ▼
 │    texture GPU
 │
 └── rendu GPU futur
         │
         ▼
        wgpu
         │
         ▼
      plateforme
```

## Socle envisagé

- langage : Rust stable ;
- fenêtre / événements : `winit` ;
- GPU : `wgpu` ;
- shaders : WGSL lorsque nécessaire ;
- framebuffer : mémoire Rust contiguë possédée par le moteur ou son contexte de rendu.

Les versions exactes sont choisies lors de M0 à partir des versions stables actuelles et consignées dans le dépôt.

## Séparation minimale des responsabilités

M0 doit permettre de distinguer conceptuellement :

### Platform

Fenêtre, événements, cycle de vie et intégration avec `winit`.

### Renderer

Surface `wgpu`, texture du framebuffer, upload et présentation.

### Framebuffer

Stockage CPU des pixels et opérations élémentaires.

### Application / Game

État appartenant au jeu et callbacks/boucle nécessaires à son évolution.

Cette séparation ne justifie pas automatiquement quatre crates, quatre traits ou une hiérarchie complexe. La structure physique doit rester aussi petite que possible.

## Performance

Rust est choisi pour obtenir une performance native de classe C/C++ avec sécurité mémoire et absence de garbage collector, et non parce que Rust serait intrinsèquement plus rapide que C++.

Principes :

1. mesurer avant d'optimiser ;
2. conserver des baselines reproductibles ;
3. surveiller allocations et copies dans les chemins chauds ;
4. privilégier des données contiguës lorsque pertinent ;
5. ne paralléliser qu'un travail mesurable et suffisamment important ;
6. utiliser le GPU lorsque le problème s'y prête, pas pour transformer chaque opération en shader.

## Politique d'abstraction

Toute abstraction importante doit répondre à au moins un de ces critères :

- elle élimine une duplication observée ;
- elle matérialise une frontière technique réelle ;
- elle est nécessaire à une fonctionnalité déjà demandée par un jeu ;
- elle permet de tester une responsabilité autrement difficile à isoler ;
- elle apporte un bénéfice mesuré.

« Cela pourrait être utile plus tard » n'est pas un critère suffisant.

## Compatibilité avec olcPixelGameEngine

La philosophie et certaines conventions d'OLC constituent une référence, pas un contrat de compatibilité binaire ou source.

Nous pouvons conserver une API familière lorsque cela améliore la simplicité ou permet de suivre un enseignement Javidx9. Nous pouvons aussi nous en écarter lorsque les idiomes Rust, la sécurité, `wgpu` ou l'expérience acquise rendent une autre conception préférable.

## Unsafe

`unsafe` n'est pas interdit, mais doit être exceptionnel :

- périmètre minimal ;
- justification documentée ;
- API sûre autour de la zone concernée ;
- absence d'`unsafe` préférable lorsqu'une solution sûre reste simple et suffisamment performante.

## Dépendances

Une dépendance doit résoudre un problème non spécifique à notre moteur mieux que nous ne le ferions raisonnablement nous-mêmes.

Inversement, les éléments pédagogiquement centraux — framebuffer, primitives de dessin, boucle moteur et certaines briques géométriques — peuvent volontairement être implémentés dans le projet afin d'en comprendre le fonctionnement.
