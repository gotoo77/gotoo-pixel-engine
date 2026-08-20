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
framebuffer, input, backend gamepad, stockage, audio et horloges. `Input` conserve
aussi l'état runtime des profils de calibration par périphérique. À chaque
redraw, le backend normalise les périphériques avec ces profils, `PlatformApp`
construit un `Frame`, appelle `Game::update`, puis présente le framebuffer.

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

`delta_time` représente du temps de simulation fourni par le runtime, pas une
dette de temps murale. Le runtime plafonne un frame de simulation à 100 ms et
réinitialise sa référence temporelle lors des transitions focus/resume. Une
suspension navigateur ou un stall long ne peut donc pas injecter plusieurs
secondes de simulation dans la frame suivante.

Le temps brut entre deux frames reste utilisé pour le diagnostic du frame time
et des FPS. Les jeux peuvent conserver des substeps lorsqu'ils en ont besoin
pour leurs collisions ou leur intégration locale, mais ils n'ont pas à définir
leur propre politique de suspension/stall.

`storage` et `audio` sont des capabilities injectées, mais il ne s'agit pas d'un
framework de capabilities. Il n'existe ni registre générique, ni conteneur
dynamique, ni système de plugins.

La calibration gamepad n'est plus une responsabilité de `Game`. Le runtime
conserve un `GamepadProfile` par périphérique dans le sous-système `Input`, et les
backends natif/Web utilisent cet état pour produire l'input normalisé. `Frame`
n'ajoute pas de champ de calibration obligatoire ; il expose uniquement
`set_gamepad_profile`, l'opération publique effectivement utilisée par le probe
et le menu standalone de Space Invaders pour régler le périphérique.

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
- borne le `delta_time` de simulation et réinitialise le timing sur focus/resume ;
- expose via `Frame::set_gamepad_profile` le réglage explicite requis par les
  consommateurs de configuration sans faire porter cette responsabilité à `Game` ;
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

`draw_line` conserve la rasterisation visible de Bresenham mais ne parcourt pas
les portions arbitrairement longues situées hors du framebuffer. Le rasteriseur
calcule directement l'intervalle des pas de l'axe majeur qui peut être visible :
une ligne x-major effectue au plus un nombre de pas proportionnel à la largeur du
framebuffer, et une ligne y-major à sa hauteur. L'axe mineur est reconstruit à la
même phase de rasterisation que le segment original, ce qui évite de modifier les
pixels de bord en reclippant puis en relançant Bresenham depuis de nouvelles
extrémités entières. Les produits intermédiaires utilisent `i128` pour rester
sûrs jusque sur des coordonnées `i32::MIN` / `i32::MAX`.

Ce durcissement reste local au framebuffer : il ne justifie ni bibliothèque de
clipping générique ni nouvelle couche Geometry2D.

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

Le sous-système conserve également, de façon interne, le profil de calibration
associé à chaque `GamepadId`. Cet état est une configuration runtime du
périphérique et ne fait pas partie du snapshot observable utilisé par le gameplay.
La mutation des états physiques reste réservée aux backends plateforme.

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

Les backends lisent le profil runtime associé au périphérique, normalisent les
valeurs brutes puis alimentent le modèle `Input` commun. Le backend natif contient
aussi les corrections nécessaires aux D-pad/hats observés sur du matériel réel.
Le profil runtime est supprimé à la déconnexion pour éviter de réutiliser une
calibration périmée si un identifiant de périphérique est recyclé.

### `gamepad_profile.rs`

Décrit la normalisation d'axes et le seuil numérique :

- centre éventuellement asymétrique ;
- inversion d'axe ;
- dead zone ;
- threshold numérique.

`AxisCalibration` et `GamepadProfile` restent les types publics de description.
Le store `GamepadId -> GamepadProfile` est interne au runtime d'input : il ne
constitue ni un `DeviceManager`, ni un registre générique de configuration.

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

### Résolution d'assets

GPE sait aujourd'hui consommer des bytes explicites, notamment via
`SoundBank::insert_wav`, mais il ne possède pas encore de stratégie générique
cross-platform pour résoudre un chemin logique d'asset en bytes.

Le besoin a été observé avec les SFX temporaires de Smart Boy Hero :

```text
sfx.json + WAV
     ↓
include_dir / compilation
     ↓
bytes
     ↓
SoundBank
```

Cette solution locale est acceptable pour SBH maintenant :

- simple ;
- robuste ;
- identique en natif et Web ;
- offline ;
- peu d'erreurs runtime.

Elle n'est pas considérée comme la stratégie définitive d'assets du moteur. Ses
limites sont claires : modifier un asset exige un rebuild, les fichiers sont
embarqués dans le binaire ou le WASM, et il n'y a pas de hot-swap, modding ou
remplacement runtime.

Le besoin futur potentiel ressemble plutôt à :

```text
path logique
   ↓
AssetSource
   ↓
bytes
```

Avec des implémentations possibles :

- natif : filesystem ;
- Web : HTTP/fetch ou bundle ;
- tests : mémoire.

Le contrat minimal d'une future abstraction devrait probablement rester au
niveau bytes : résoudre un chemin logique, retourner des bytes ou une erreur
explicite, et laisser les consommateurs spécialisés valider le format
(`Audio` pour WAV, futur décodeur image pour sprites, parseur de maps, etc.).
Elle ne devrait pas devenir un `AssetManager` global mêlant cache, décodage,
hot-reload, formats, volumes, streaming et politiques de packaging sans besoin
démontré.

Évaluation actuelle : le besoin n'est pas encore suffisamment démontré par
plusieurs consommateurs pour justifier une abstraction moteur. SBH est un premier
signal, mais il ne couvre qu'un mapping déclaratif audio vers des WAV courts. La
préférence architecturale reste donc : assets embarqués par défaut aujourd'hui,
configuration déclarative locale lorsque c'est utile, puis réévaluation du
chargement runtime lorsqu'un second type d'asset concret, par exemple sprites,
maps, dialogues ou musiques, réclamera lui aussi une résolution de chemins
cross-platform.

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

`VirtualPad` possède les tests de sa propre mécanique tactile : cycle des
contacts, déplacements entre boutons, multi-contact, ordre des actions et reset.
Les jeux consommateurs testent leur câblage vers leurs actions, pas une seconde
implémentation locale de cette mécanique.

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

Snake a validé la séparation initiale, désormais matérialisée localement par deux
fichiers aux responsabilités distinctes :

```text
examples/snake/world.rs
    SnakeWorld, grille, serpent, nourriture, collisions, file de virages
    aucune dépendance à gotoo-pixel-engine

examples/snake/game.rs
    adaptation input, timing, layout, HUD, rendu, stockage, audio

gotoo-pixel-engine
    plateforme, rendu, input, viewport, stockage, audio
```

Le tactile de Snake passe par le `VirtualPad` partagé. Snake teste son câblage des
zones vers ses quatre actions et ses règles propres ; la mécanique générique de
contacts tactiles est testée dans `VirtualPad` lui-même. Aucun `WorldSystem`,
scene framework ou autre abstraction moteur n'a été créé pour ce découpage.

Tetris, Space Invaders, Pong, Breakout et Smart Boy Hero ont ensuite validé
d'autres besoins : menus, gamepad, tactile, audio, deux joueurs, collisions,
feedback et micro-puzzle tour par tour à monde pur.

Le probe gamepad et le menu standalone de Space Invaders sont les consommateurs
concrets de `Frame::set_gamepad_profile`. Les autres jeux lisent simplement
l'`Input` normalisé ; Arcade et `PauseGame` n'ont donc aucune raison de connaître
ou relayer la calibration du périphérique.

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
