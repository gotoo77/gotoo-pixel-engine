# Void Canticle — Backgrounds : notes d’intégration VC3.3

**Base :** `void-canticle-v3.3`  
**Branche :** `design/void-canticle-backgrounds`  
**But de cette tranche :** transformer le dossier de DA en un premier consommateur jouable, sans généralisation prématurée.

## État constaté au checkpoint

VC3.3 a déjà posé la bonne séparation technique :

- front/presentation HD en **540×960** ;
- gameplay rendu dans un framebuffer séparé ;
- compositing du gameplay au-dessus d’un background HD ;
- fond gameplay actuel généré procéduralement (`gameplay_background.rs`) ;
- événement de singularité, haze et étoiles également procéduraux.

Cette fondation est cohérente avec la DA du pack. Le manque actuel n’est donc pas un nouveau renderer : c’est **un vrai asset de décor**, puis la validation de sa lisibilité en situation.

## Slice B1 — fond authored réel

Cette branche introduit un seul asset de production :

```text
assets/void_canticle/backgrounds/
└── void_abyss/
    └── background.png   # source authored 180×320, dérivée du concept VOID / ABYSS
```

Le runtime décode cette image une fois au démarrage, la met à l’échelle vers le framebuffer HD 540×960, puis conserve ce framebuffer. Chaque frame, le fond est copié avant le compositing du gameplay.

Ce choix garde volontairement la surface technique minimale :

```text
PNG authored
    ↓
Image::decode_png (180×320)
    ↓
upscale bilinéaire au chargement
    ↓
Framebuffer 540×960
    ↓
gameplay compositor existant
```

## Ce qui n’est volontairement pas fait

Pas encore de :

- `BackgroundManager` ;
- système de biome/stage ;
- JSON de configuration ;
- parallaxe générique ;
- streaming d’assets ;
- hot reload ;
- mélange automatique de couches ;
- animation de décor pilotée par données.

Ces abstractions devront être méritées par au moins deux ou trois backgrounds réellement jouables.

## Concepts vs production

Les huit images du pack sont rangées sous :

```text
docs/design/art/void_canticle_backgrounds/concepts/
```

Ce sont des **concept arts de référence**, pas des assets runtime contractuels.

Les copies dans `concepts/` sont des miniatures JPEG 120×213 compressées pour revue Git et documentation ; les sources haute définition restent dans le pack d’origine. Les noms sont normalisés sur les huit biomes :

- `blue_silence`
- `void_abyss`
- `dead_empire`
- `graveyard`
- `toxic_canticle`
- `ashen_empire`
- `blood_canticle`
- `eclipse`

Un concept ne passe dans `assets/void_canticle/backgrounds/` qu’au moment où il devient un vrai consommateur gameplay.

## Validation attendue de B1

Avant de passer à la parallaxe :

1. lancer une partie complète ;
2. vérifier le joueur, les projectiles hostiles et les pickups pendant les pics de densité ;
3. vérifier les trois châssis ;
4. vérifier les annonces, boss et effets de Canticle ;
5. confirmer que les détails brillants du décor ne sont jamais pris pour des bullets ;
6. comparer visuellement le nouveau `VOID / ABYSS` au fond procédural VC3.3.

Si la lecture est insuffisante, corriger **l’asset de production** (luminosité locale, contraste, masque du corridor) avant de complexifier le renderer.

## Slice suivante proposée : B2

Seulement après validation de B1 :

- extraire 2–3 couches simples d’un second environnement, idéalement `GRAVEYARD` ;
- introduire quelques débris décoratifs à vitesses indépendantes ;
- conserver le fond authored principal statique ou quasi statique ;
- mesurer le coût avant toute abstraction supplémentaire.

`GRAVEYARD` est un bon second consommateur parce que son champ d’épaves fournit un besoin réel de couches séparées. C’est à ce moment qu’une petite API de parallaxe pourra éventuellement émerger.
