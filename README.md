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

## M1.2 — Primitives de dessin CPU

Les primitives CPU disponibles sur `Framebuffer` ignorent les pixels hors framebuffer :

- `draw_line(x0, y0, x1, y1, pixel)` ;
- `draw_rect(x, y, width, height, pixel)` ;
- `fill_rect(x, y, width, height, pixel)` ;
- `draw_circle(center_x, center_y, radius, pixel)` ;
- `fill_circle(center_x, center_y, radius, pixel)`.

Politique de clipping M1.2 : il n'y a pas de système général de clipping. Les primitives restent locales à `Framebuffer`, calculent leurs bornes en entier large, rejettent les formes dont la boîte englobante ne croise pas le framebuffer et clampent les spans/remplissages à la zone visible. Un cercle plein qui couvre tout le framebuffer remplit directement la zone visible. Un contour de cercle n'est rejeté comme invisible que lorsque son rayon est très au-delà de toute coordonnée visible selon un test conservateur.

Baseline reproductible :

```bash
cargo bench --bench primitives
```

## M1.3 — Input, timing et exemple public

Un jeu implémente `Game::update` et retourne `GameResult::Continue` ou `GameResult::Exit`. Le moteur fournit à chaque frame un `Frame` contenant le framebuffer, l'état d'entrée et `delta_time`. `Escape` n'a pas de sémantique moteur spéciale : un jeu choisit lui-même d'en faire une sortie ou non.

```rust
use gotoo_pixel_engine::{
    run, EngineConfig, Frame, Game, GameResult, Key, MouseButton, Pixel,
};

struct MiniGame {
    x: f32,
    y: f32,
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

        let color = if frame.input.mouse_button(MouseButton::Left).held() {
            Pixel::RED
        } else {
            Pixel::WHITE
        };

        frame.framebuffer.clear(Pixel::BLACK);
        frame.framebuffer.fill_circle(self.x.round() as i32, self.y.round() as i32, 8, color);

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
        MiniGame { x: 160.0, y: 90.0 },
    )?;

    Ok(())
}
```

## M1.5 — Démo WebAssembly

La démo Web réutilise le même `DemoGame` que le chemin natif et se lance dans un navigateur via WebAssembly/WebGPU.

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --example web_demo
wasm-bindgen --target web \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/debug/examples/web_demo.wasm
python3 -m http.server 8000 --directory web
```

Ouvrir ensuite <http://127.0.0.1:8000>.

M1.5 a été validé manuellement sous Chromium et Firefox desktop : rendu framebuffer, clavier, souris, clic gauche et `Escape`.

Voir [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md) et [REFERENCES.md](REFERENCES.md).
