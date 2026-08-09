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

Le **spike technique minimal (M0)** est terminé : ouvrir une fenêtre, maintenir un framebuffer CPU, l'envoyer au GPU et gérer une boucle interactive avec clavier et delta-time.

Le travail en cours transforme progressivement ce spike en petite API publique, en commençant par **M1.1 — Pixel et framebuffer public**.

## M0 — Lancer le spike plateforme

```bash
cargo run
```

La démonstration ouvre une fenêtre `960x540` qui affiche un framebuffer CPU `320x180` uploadé vers une texture GPU. La touche `Espace` bascule la palette de la démo, `Echap` ou la fermeture de fenêtre quitte l'application. Le titre de la fenêtre affiche une mesure simple du temps de frame et du FPS pour établir une première baseline.

## Versions M0

- Rust : `1.97.1`, pinné dans `rust-toolchain.toml`.
- `winit` : `0.30.13`.
- `wgpu` : `30.0.0`.
- `pollster` : `1.0.1`, utilisé uniquement pour initialiser `wgpu` depuis la boucle synchrone `winit`.

Le chemin CPU framebuffer vers GPU vers écran se lit directement dans `src/demo.rs`, `src/framebuffer.rs` et `src/renderer.rs`.

## M1.1 — Pixel et framebuffer public

L'API publique actuelle expose :

- `Pixel::rgb(r, g, b)` et `Pixel::rgba(r, g, b, a)` ;
- quelques couleurs constantes : `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `TRANSPARENT` ;
- `Framebuffer::new(width, height)` ;
- `Framebuffer::clear(pixel)` ;
- `Framebuffer::draw(x, y, pixel)`, borné, qui retourne `false` hors framebuffer ;
- `Framebuffer::pixel(x, y)`, borné, qui retourne `None` hors framebuffer ;
- `Framebuffer::as_rgba8()` pour le chemin d'upload GPU.

Voir [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md) et [REFERENCES.md](REFERENCES.md).
