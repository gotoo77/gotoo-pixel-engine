# Architecture — gotoo-pixel-engine

## Philosophie

Le moteur doit rester suffisamment petit pour qu'un développeur puisse suivre le
chemin complet entre son code de jeu et le pixel affiché, l'entrée reçue, la
valeur sauvegardée ou le son joué.

L'architecture actuelle privilégie :

- une API publique courte ;
- un framebuffer CPU comme primitive centrale ;
- des frontières plateforme explicites ;
- des backends natif/Web cachés au jeu ;
- des abstractions ajoutées seulement après besoin observé ;
- la réutilisation des primitives déjà présentes avant d'en ajouter de nouvelles.

## Flux principal

```text
OS / navigateur
        ↓
winit / événements
        ↓
PlatformApp
        ↓
Input + gamepad + timing + capabilities
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

`PlatformApp` possède les ressources longue durée : fenêtre, renderer,
framebuffer, input, backend gamepad, stockage, audio et horloges. À chaque
redraw, il met à jour les périphériques, construit un `Frame`, appelle
`Game::update`, puis présente le framebuffer.

## `Game` et `Frame`

`Game` reste le contrat central :

```text
Game::update(&mut self, &mut Frame) -> GameResult
```

`Frame` agrège explicitement les ressources disponibles pendant une frame :

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

`storage` et `audio` sont des capabilities injectées, mais il ne s'agit pas d'un
framework de capabilities. Il n'existe ni registre générique, ni conteneur
dynamique, ni système de plugins.

`Game::gamepad_profile()` existe actuellement comme point d'injection de profils
de périphériques. Cette responsabilité est considérée comme provisoire : la
composition Arcade a montré que la calibration est liée au périphérique/runtime
plus qu'au gameplay lui-même.

## Modules moteur

### `lib.rs`

Surface publique du crate. Elle réexporte notamment :

- `run`, `EngineConfig`, `Game`, `Frame`, `GameResult` ;
- `Pixel`, `Framebuffer` ;
- `Input`, `Key`, `MouseButton`, `Touch`, `TouchPhase`, `ButtonState` ;
- `GamepadId`, `GamepadButton`, informations de connexion ;
- `ActionId`, `ControlBinding`, `ControlMap` ;
- `AxisCalibration`, `GamepadProfile` ;
- `Size`, `Rect`, `Viewport` ;
- `LocalStorage`, `NoopStorage` ;
- `Audio`, `SoundBank`, `SoundId`, `NoopAudio` ;
- les primitives UI minimales via `ui`.

### `platform.rs`

Frontière principale avec `winit`. Elle :

- crée la fenêtre et la boucle d'événements ;
- initialise le renderer ;
- convertit clavier, souris et tactile vers `Input` ;
- poll le backend gamepad ;
- active l'audio après interaction utilisateur ;
- injecte stockage et audio dans `Frame` ;
- gère resize, focus et raccourcis plateforme.

Le jeu ne manipule pas directement `winit`.

### `renderer.rs`

Responsable du chemin GPU :

- création surface/device/queue `wgpu` ;
- texture GPU correspondant au framebuffer CPU ;
- upload RGBA8 ;
- shader WGSL de présentation ;
- conservation sRGB lorsque la surface le permet, encodage shader sinon ;
- application du viewport ;
- présentation de la surface.

Le renderer reste interne.

### `framebuffer.rs`

Primitive publique pixel-first. Elle stocke les pixels en RGBA8 contigu et
fournit :

- effacement ;
- accès pixel borné ;
- lignes ;
- rectangles ;
- cercles ;
- texte bitmap ;
- accès `as_rgba8` pour l'upload GPU.

Le framebuffer est volontairement un renderer logiciel simple, pas une API de
scène.

### `viewport.rs`

Expose `Size`, `Rect` et `Viewport`.

`Viewport::new(surface_size, framebuffer_size)` conserve le ratio logique du
framebuffer et calcule letterboxing/pillarboxing. Le même viewport sert au rendu
et au mapping souris/tactile.

`Rect` contient également les petites opérations géométriques déjà démontrées,
notamment `contains()` et `intersects()`. La règle est de réutiliser ces
primitives avant d'envisager une bibliothèque Geometry2D plus large.

### `input.rs`

État public en lecture des entrées :

- clavier minimal ;
- souris ;
- tactile brut par événements de frame ;
- gamepads connectés ;
- états `pressed`, `held`, `released` ;
- événements connexion/déconnexion gamepad.

La mutation appartient aux backends plateforme et reste crate-private.

### `control.rs`

`ControlMap` fait converger plusieurs sources physiques vers des actions de jeu :

```text
Key ------------------┐
Gamepad any ----------┼──> ActionId -> ButtonState
Gamepad device -------┤
Virtual/touch --------┘
```

Cette abstraction est utilisée par plusieurs jeux et par l'UI. Elle ne cherche
pas à gérer profils utilisateurs, remapping persistant ou système de commandes
générique tant qu'un besoin concret ne les impose pas.

### `gamepad.rs` / `gamepad/browser.rs`

Backends gamepad :

- natif : `gilrs` ;
- Web : Gamepad API via `web-sys` pour les mappings standards.

Les backends traduisent les périphériques vers le modèle `Input` commun. Le
backend natif contient aussi les corrections nécessaires aux D-pad/hats observés
sur du matériel réel.

### `gamepad_profile.rs`

Décrit la normalisation d'axes et le seuil numérique :

- centre éventuellement asymétrique ;
- inversion d'axe ;
- dead zone ;
- threshold numérique.

Le profil est un mécanisme de normalisation de périphérique, pas une abstraction
de gameplay.

### `storage.rs`

Capability key/value minimale :

```text
Game
  -> Frame.storage: dyn LocalStorage
       native: fichier local utilisateur
       web: window.localStorage
```

`NoopStorage` permet les tests. Les erreurs sont retournées au jeu mais ne font
pas tomber le runtime par défaut.

### `audio.rs`

Capability audio one-shot minimale :

```text
Game
  -> Frame.audio: dyn Audio
       native: rodio/cpal
       web: WebAudio
```

`SoundBank` garde les WAV du jeu et les enregistre paresseusement dans le backend.
Le sous-ensemble accepté reste volontairement restreint à des WAV PCM 16-bit
mono/stéréo en 44100 ou 48000 Hz.

Le moteur ne fournit pas encore de mixer public, streaming ou pipeline d'assets.

### `ui`

UI immediate-mode minimale, introduite uniquement après duplication observée :

- panneaux ;
- texte centré ;
- items de menu ;
- `MenuState` ;
- helpers de navigation ;
- `VirtualPad` ;
- wrapper `PauseGame`.

Ce module n'est pas un framework UI généraliste. Il n'existe pas d'arbre de
widgets, de callbacks, de système de focus ou de thème générique.

## Frontières plateforme

Native :

- fenêtre/événements : `winit` ;
- rendu : `wgpu` ;
- gamepad : `gilrs` ;
- stockage : filesystem via `directories` ;
- audio : `rodio`/`cpal`.

Web/WASM :

- point d'entrée : `wasm-bindgen` ;
- fenêtre/canvas/événements : `winit` Web ;
- rendu : `wgpu` WebGPU ;
- gamepad : Gamepad API via `web-sys` ;
- stockage : `localStorage` ;
- audio : WebAudio.

Le code de jeu ne contient normalement pas de `cfg(wasm32)`, d'accès DOM, de
filesystem, de `rodio`, `gilrs` ou `web_sys`.

## Consommateurs architecturaux

Snake a validé la séparation initiale :

```text
SnakeWorld
    métier pur

SnakeGame
    adaptation input, layout, HUD, storage, audio

gotoo-pixel-engine
    plateforme, rendu, input, viewport, storage, audio
```

Tetris, Space Invaders, Pong et Breakout ont ensuite validé d'autres besoins :
menus, gamepad, tactile, audio, deux joueurs, collisions et feedback.

`GPE Arcade` joue un rôle différent : il compose plusieurs jeux dans un même
runtime. Il sert donc de test de cohérence des frontières entre « jeu
réutilisable » et « entrypoint standalone ». Les incohérences qu'il révèle sont
des candidats prioritaires à la consolidation locale avant toute nouvelle
abstraction moteur.

## Politique d'abstraction

Une abstraction importante doit répondre à au moins un de ces critères :

- elle élimine une duplication observée ;
- elle matérialise une frontière technique réelle ;
- elle est nécessaire à une fonctionnalité demandée par un jeu ;
- elle permet de tester une responsabilité autrement difficile à isoler ;
- elle apporte un bénéfice mesuré.

Une abstraction déjà entrée dans le moteur doit également être utilisée par ses
consommateurs. Si plusieurs jeux la contournent, il faut vérifier soit sa
découvrabilité, soit son placement, soit sa pertinence.

« Cela pourrait être utile plus tard » n'est pas un critère suffisant.

## Dépendances

Une dépendance doit résoudre un problème non spécifique au moteur mieux que nous
ne le ferions raisonnablement nous-mêmes.

Les briques pédagogiquement centrales restent implémentées dans le projet :
framebuffer, primitives CPU, boucle moteur, mapping logique et contrôles. Les
frontières spécialisées utilisent des crates ou APIs dédiées lorsqu'une
implémentation maison serait surtout fragile : `wgpu`, `winit`, `gilrs`,
`directories`, `rodio`, Web APIs.

## Unsafe

`unsafe` n'est pas interdit, mais doit rester exceptionnel :

- périmètre minimal ;
- justification documentée ;
- API sûre autour de la zone concernée ;
- solution sûre préférée lorsqu'elle reste simple et suffisamment performante.
