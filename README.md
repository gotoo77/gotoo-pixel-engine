# gotoo-pixel-engine

`gotoo-pixel-engine` est un moteur de jeu **pixel-first en Rust**, inspiré par la
philosophie de l'`olcPixelGameEngine`, conçu pour apprendre, expérimenter et
construire de petits jeux en gardant le chemin complet lisible.

Le projet ne cherche pas à devenir un moteur généraliste. Son objectif reste une
boucle simple :

```text
initialiser
    -> lire les entrées
    -> mettre à jour l'état du jeu
    -> dessiner dans un framebuffer CPU
    -> présenter le résultat
```

Principe directeur :

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce
> qu'elle pourrait être utile plus tard.

Pas d'ECS, de scene graph, d'asset manager ou de framework UI tant qu'un besoin
observé dans plusieurs consommateurs réels ne les justifie pas.

## État actuel

Le moteur expose aujourd'hui une API publique volontairement courte pour écrire
un jeu pixel-first natif et Web/WASM sans manipuler directement `winit`, `wgpu`,
le filesystem, le DOM, WebAudio ou les APIs gamepad plateforme.

Capacités disponibles :

- framebuffer CPU RGBA8 ;
- primitives de dessin : pixels, lignes, rectangles, cercles et remplissages ;
- texte bitmap intégré ;
- clavier, souris et tactile brut ;
- gamepad natif et Web avec boutons, D-pad et stick gauche normalisés ;
- états `pressed`, `held`, `released` ;
- `ControlMap` pour faire converger clavier, gamepad ciblé/global et contrôles virtuels ;
- profils et calibration gamepad par périphérique ;
- timing de simulation par frame via `delta_time`, borné face aux stalls et suspensions ;
- viewport conservant le ratio du framebuffer ;
- mapping cohérent surface -> viewport -> framebuffer pour souris/tactile ;
- stockage local persistant natif/Web via `LocalStorage` ;
- audio one-shot et boucles identifiables natif/Web via `Audio` et `SoundBank` ;
- UI immediate-mode minimale : panneaux, texte centré, menus, contrôles virtuels et pause ;
- backend natif via `winit`/`wgpu` ;
- cible WebAssembly/WebGPU.

## Jeux consommateurs

Le moteur est exercé par plusieurs jeux réels :

- Snake ;
- Tetris ;
- Space Invaders ;
- Pong deux joueurs ;
- Breakout ;
- Smart Boy Hero ;
- `GPE Arcade`, qui compose plusieurs jeux dans un même runtime et sert aussi de
  test architectural multi-jeux.

Version publique de l'Arcade :

<https://gotoo77.github.io/gotoo-pixel-engine/>

Les jeux Web restent également accessibles individuellement via `snake.html`,
`tetris.html`, `space_invaders.html`, `pong.html`, `breakout.html`,
`smart_boy_hero.html` et `smart_boy_hero_iso.html`.

## Exemple minimal

```rust
use gotoo_pixel_engine::{
    run, EngineConfig, Frame, Game, GameResult, Key, Pixel,
};

struct MiniGame {
    x: f32,
}

impl Game for MiniGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        let speed = 120.0 * frame.delta_time.as_secs_f32();
        if frame.input.key(Key::Left).held() {
            self.x -= speed;
        }
        if frame.input.key(Key::Right).held() {
            self.x += speed;
        }

        frame.framebuffer.clear(Pixel::BLACK);
        frame.framebuffer.fill_circle(
            self.x.round() as i32,
            90,
            8,
            Pixel::WHITE,
        );

        GameResult::Continue
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        EngineConfig {
            title: "mini game".into(),
            framebuffer_width: 320,
            framebuffer_height: 180,
            window_width: 960,
            window_height: 540,
        },
        MiniGame { x: 160.0 },
    )?;

    Ok(())
}
```

## Commandes utiles

Les commandes de développement sont centralisées dans `scripts/dev.py`. Elles
sont utilisables sous Windows, Linux et macOS dès que Python 3 est disponible ;
les scripts `.sh` restent des wrappers de compatibilité Unix.

Lancer le sélecteur de jeux natif :

```bash
python scripts/dev.py run-game
```

Lancer directement un jeu :

```bash
python scripts/dev.py run-game snake
python scripts/dev.py run-game smart-boy-hero --release
```

Construire tous les entrypoints Web/WASM :

```bash
rustup target add wasm32-unknown-unknown
python scripts/dev.py check-web
```

Construire les paquets Web locaux avec `wasm-bindgen` :

```bash
python scripts/dev.py build-web
```

Puis servir le dossier `web` :

```bash
python scripts/dev.py serve-web
```

## Validation

Validation native complète :

```bash
python scripts/dev.py check
```

Validation de tous les entrypoints Web :

```bash
python scripts/dev.py check-web
```

La CI GitHub utilise la même CLI Python pour les validations natives/Web et le
packaging JavaScript/WASM. Le workflow GitHub Pages appelle également
`scripts/dev.py build-web --pages` pour construire le bundle release et
assembler `dist/`.

## Licence

Le code propre à `gotoo-pixel-engine` est distribué sous licence MIT. Voir
[`LICENSE`](LICENSE). Les références externes, inspirations et éventuels ports
restent documentés séparément dans [`REFERENCES.md`](REFERENCES.md), avec leurs
propres obligations de licence et de provenance.

Voir aussi [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md) et
[REFERENCES.md](REFERENCES.md).
