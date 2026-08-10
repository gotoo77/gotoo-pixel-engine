# gotoo-pixel-engine

`gotoo-pixel-engine` est un moteur de jeu **pixel-first en Rust**,
inspire par la philosophie de l'`olcPixelGameEngine`, concu pour apprendre,
experimenter et construire de petits jeux en gardant le chemin complet lisible.

Le projet ne cherche pas a devenir un moteur generaliste. Son objectif est une
boucle simple :

```text
initialiser
    -> lire les entrees
    -> mettre a jour l'etat du jeu
    -> dessiner dans un framebuffer CPU
    -> presenter le resultat
```

Principe directeur :

> Une abstraction entre dans le moteur parce qu'un jeu en a besoin, pas parce
> qu'elle pourrait etre utile plus tard.

Pas d'ECS, de scene graph, d'asset manager ou de framework UI tant qu'un besoin
observe dans un jeu reel ne les justifie pas.

## Etat Actuel

Le moteur expose aujourd'hui une petite API publique suffisante pour ecrire un
jeu pixel-first natif et Web/WASM sans manipuler directement `winit`, `wgpu`, le
filesystem, le DOM ou l'audio plateforme.

Capacites disponibles :

- framebuffer CPU RGBA8 ;
- primitives de dessin : pixels, lignes, rectangles, cercles, remplissages ;
- texte bitmap integre ;
- clavier, souris et tactile brut ;
- etats `pressed`, `held`, `released` pour les boutons ;
- timing par frame via `delta_time` ;
- viewport conservant le ratio du framebuffer ;
- mapping coherent surface -> viewport -> framebuffer pour souris/tactile ;
- stockage local persistant natif/Web via `LocalStorage` ;
- audio one-shot natif/Web via `Audio` ;
- backend natif via `winit`/`wgpu` ;
- cible WebAssembly/WebGPU ;
- deploiement Snake sur GitHub Pages.

Snake est le premier jeu reel de validation. Il couvre actuellement :

- monde Snake 32x18 avec score ;
- HUD `SCORE n    BEST m` ;
- BEST persistant localement ;
- replay clavier/souris/tactile ;
- layout clavier et layout tactile ;
- D-pad tactile ;
- audio `snake.eat` et `snake.death` ;
- version native et version Web publiee.

Version publique Snake :

<https://gotoo77.github.io/gotoo-pixel-engine/snake.html>

## Exemple Minimal

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

`Frame` contient aussi `storage`, `audio`, `surface_size` et `viewport`. Un jeu
les utilise seulement lorsqu'il en a besoin.

## Commandes Utiles

Lancer la demo native :

```bash
cargo run
```

Lancer Snake natif :

```bash
cargo run --example snake
```

Construire Snake Web/WASM :

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --example snake_web
```

Generer `web/pkg` avec `wasm-bindgen` :

```bash
wasm-bindgen --target web \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/debug/examples/snake_web.wasm
```

Servir le dossier Web localement :

```bash
python3 -m http.server 8000 --directory web
```

Puis ouvrir :

- <http://127.0.0.1:8000/snake.html> pour Snake Web.

La page `web/index.html` reste la page de demo Web historique ; elle necessite
de generer `web/pkg/web_demo.js` a partir de l'exemple `web_demo`.

## Validation

Commandes de validation courantes :

```bash
git diff --check
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo check --examples
cargo build --target wasm32-unknown-unknown --example snake_web
```

Voir aussi [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md) et
[REFERENCES.md](REFERENCES.md).
