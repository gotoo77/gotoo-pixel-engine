# gotoo-pixel-engine

Moteur de jeu **pixel-first en Rust**, inspiré par la philosophie de l'`olcPixelGameEngine`, conçu pour apprendre, expérimenter et créer.

## Intention

`gotoo-pixel-engine` n'a pas vocation à devenir un moteur généraliste concurrent de Godot, Bevy, Unity ou Unreal.

L'objectif est de conserver une boucle de développement volontairement simple et compréhensible :

```text
initialiser
    ↓
lire les entrées
    ↓
mettre à jour l'état du jeu
    ↓
dessiner
    ↓
recommencer
```

Le projet privilégie :

- une API petite et explicite ;
- Rust natif, sans couche C++ imposée ;
- un framebuffer CPU simple comme primitive fondamentale ;
- un backend GPU moderne via `wgpu` ;
- des performances mesurées plutôt que supposées ;
- l'apprentissage du fonctionnement d'un moteur plutôt que son occultation ;
- l'ajout de fonctionnalités à partir de besoins de jeux réels.

## Principe directeur

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce qu'un développeur pourrait éventuellement en avoir besoin un jeudi de novembre 2031.

Pas d'ECS, de scene graph, de système de plugins, de scripting ou d'éditeur tant qu'un besoin concret ne les justifie pas.

## État

Le projet démarre par un **spike technique minimal (M0)** : ouvrir une fenêtre, maintenir un framebuffer CPU, l'envoyer au GPU et gérer une boucle interactive avec clavier et delta-time.

Voir [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md) et [REFERENCES.md](REFERENCES.md).
