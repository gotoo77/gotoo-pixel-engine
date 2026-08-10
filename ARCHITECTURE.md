# Architecture — gotoo-pixel-engine

## Philosophie

Le moteur doit rester suffisamment petit pour qu'un developpeur puisse suivre le
chemin complet entre son code de jeu et le pixel affiche, l'entree recue, la
valeur sauvegardee ou le son joue.

L'architecture actuelle privilegie :

- une API publique courte ;
- un framebuffer CPU comme primitive centrale ;
- des frontieres plateforme explicites ;
- des backends natif/Web caches au jeu ;
- des abstractions ajoutees seulement apres besoin observe.

## Flux Principal

```text
OS / navigateur
        ↓
winit / evenements
        ↓
PlatformApp
        ↓
Input / timing / capabilities
        ↓
Frame
        ↓
Game::update
        ↓
Framebuffer CPU
        ↓
Renderer
        ↓
wgpu
        ↓
surface
```

`PlatformApp` possede les ressources longues durees : fenetre, renderer,
framebuffer, input, stockage, audio et horloges. A chaque redraw, il construit
un `Frame`, appelle `Game::update`, puis demande au renderer de presenter le
framebuffer.

## Frame

`Frame` est l'agregat explicite des ressources disponibles pendant une frame :

```text
Frame
 ├── framebuffer
 ├── input
 ├── delta_time
 ├── surface_size
 ├── viewport
 ├── storage
 └── audio
```

`storage` et `audio` peuvent etre decrits comme des capabilities injectees dans
la frame, mais il ne s'agit pas d'un framework de capabilities. Il n'existe pas
de registre generique, de conteneur dynamique ou de systeme de plugins. Chaque
champ reste explicite et justifie par un besoin de jeu deja rencontre.

## Modules

### `lib.rs`

Surface publique du crate. Reexporte les types utiles aux jeux :

- `run`, `EngineConfig`, `Game`, `Frame`, `GameResult` ;
- `Pixel`, `Framebuffer` ;
- `Input`, `Key`, `MouseButton`, `Touch`, `TouchPhase`, `ButtonState` ;
- `Size`, `Rect`, `Viewport` ;
- `LocalStorage`, `NoopStorage` ;
- `Audio`, `SoundId`, `NoopAudio`.

### `platform.rs`

Frontiere principale avec `winit`. Elle :

- cree la fenetre et la boucle d'evenements ;
- initialise le renderer ;
- convertit les evenements clavier/souris/tactile vers `Input` ;
- active l'audio sur interaction utilisateur ;
- injecte `LocalStorage` et `Audio` dans `Frame` ;
- gere le resize et la perte de focus.

Le jeu ne manipule pas directement `winit`.

### `renderer.rs`

Responsable du chemin GPU :

- creation surface/device/queue `wgpu` ;
- texture GPU du framebuffer CPU ;
- upload RGBA8 ;
- shader WGSL de presentation ;
- conservation sRGB lorsque la surface le permet, encodage shader sinon ;
- application du viewport GPU ;
- presentation de la surface.

Le renderer reste interne.

### `framebuffer.rs`

Primitive publique pixel-first. Stocke les pixels en RGBA8 contigu et fournit :

- effacement ;
- pixel borne ;
- lignes ;
- rectangles ;
- cercles ;
- texte bitmap ;
- acces `as_rgba8` pour l'upload GPU.

Les primitives ignorent ou clippent les coordonnees hors framebuffer.

### `input.rs`

Etat public de lecture des entrees :

- clavier minimal ;
- souris minimale ;
- tactile brut par evenements de frame ;
- etats `pressed`, `held`, `released`.

La mutation de l'input est crate-private et appartient a la plateforme.

### `viewport.rs`

Primitive publique de presentation logique :

- `Size` ;
- `Rect` ;
- `Viewport`.

`Viewport::new(surface_size, framebuffer_size)` conserve le ratio du framebuffer,
calcule un rectangle de presentation et produit du letterboxing ou pillarboxing
si necessaire. Le mapping `map_surface_position` transforme une position de
surface en coordonnees framebuffer. Une position situee dans les bandes hors
viewport retourne `None`.

Le meme viewport est utilise par le rendu et par la conversion souris/tactile.

### `storage.rs`

Capability key/value minimale :

```text
Game
  -> Frame.storage: dyn LocalStorage
       native: fichier local utilisateur
       web: window.localStorage
```

Le backend natif utilise `directories` pour choisir un repertoire utilisateur
approprie, puis un fichier texte par cle. Le backend Web utilise
`localStorage`. `NoopStorage` permet les tests ou l'absence de persistance.

Les erreurs sont retournees au jeu mais ne doivent pas empecher le moteur de
fonctionner. Snake les ignore volontairement pour garder `BEST = 0` ou conserver
la valeur memoire.

### `audio.rs`

Capability audio one-shot minimale :

```text
Game
  -> Frame.audio: dyn Audio
       native: rodio/cpal
       web: WebAudio
```

L'API publique permet d'enregistrer un WAV embarque par `SoundId`, puis de le
jouer. Le sous-ensemble accepte est volontairement reduit : PCM 16-bit,
mono/stereo, 44100 ou 48000 Hz.

Le backend Web cree ou reprend un `AudioContext` et doit etre active apres une
interaction utilisateur, conformement aux contraintes navigateur. Le jeu ne
connait pas WebAudio. `NoopAudio` permet les tests.

Les erreurs audio sont non bloquantes pour le jeu.

## Frontieres Plateforme

Native :

- fenetre/evenements : `winit` ;
- rendu : `wgpu` ;
- stockage : filesystem via `directories` ;
- audio : `rodio`/`cpal`.

Web/WASM :

- point d'entree : `wasm-bindgen` ;
- fenetre/canvas/evenements : `winit` web ;
- rendu : `wgpu` WebGPU ;
- stockage : `localStorage` via `web-sys` ;
- audio : WebAudio via `web-sys`.

Ces details restent dans le moteur ou dans les points d'entree Web. Le code de
jeu ne contient pas de `cfg(wasm32)`, d'acces DOM, de filesystem, de `rodio` ou
de `web_sys`.

## Snake Comme Validation

Snake a servi de premier test architectural complet.

```text
SnakeWorld
    metier pur
    grille, serpent, nourriture, score, collisions, directions

SnakeGame
    adaptation/presentation
    input clavier/tactile, layout, HUD, replay, BEST, audio

gotoo-pixel-engine
    plateforme, rendu, input, viewport, storage, audio
```

`SnakeWorld` ne connait ni le framebuffer, ni le viewport, ni le stockage, ni
l'audio, ni les controles tactiles. Les inputs convergent vers des directions,
puis vers `SnakeWorld::queue_direction`.

Cette separation est le niveau d'abstraction souhaite a ce stade : assez nette
pour tester le moteur, pas assez generale pour justifier un ECS, une UI ou un
asset manager.

## Politique D'abstraction

Une abstraction importante doit repondre a au moins un de ces criteres :

- elle elimine une duplication observee ;
- elle materialise une frontiere technique reelle ;
- elle est necessaire a une fonctionnalite demandee par un jeu ;
- elle permet de tester une responsabilite autrement difficile a isoler ;
- elle apporte un benefice mesure.

« Cela pourrait etre utile plus tard » n'est pas un critere suffisant.

## Dependances

Une dependance doit resoudre un probleme non specifique au moteur mieux que nous
ne le ferions raisonnablement nous-memes.

Les briques pedagogiquement centrales restent implementees dans le projet :
framebuffer, primitives CPU, boucle moteur et mapping logique. Les frontieres
specialisees utilisent des crates ou APIs dediees lorsque cela evite une
implementation fragile : `wgpu`, `winit`, `directories`, `rodio`, Web APIs.

## Unsafe

`unsafe` n'est pas interdit, mais doit rester exceptionnel :

- perimetre minimal ;
- justification documentee ;
- API sure autour de la zone concernee ;
- absence d'`unsafe` preferee lorsqu'une solution sure reste simple et assez
  performante.
