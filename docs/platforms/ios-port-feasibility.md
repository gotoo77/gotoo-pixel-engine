# GPE — étude de faisabilité d'un backend iOS natif

- **Statut** : étude d'architecture, aucun backend iOS implémenté
- **Date** : 2026-08-20
- **Snapshot audité** : `main@4c28829da37651a329b4f4e0a3e58ddd93b14668`
- **Portée** : iPhone/iPad natifs, pas une WebView et pas le backend Web/WASM
- **Décision** : **faisable, sous réserve d'un spike Xcode/simulateur**

## Résumé exécutif

Le portage natif iOS de Gotoo Pixel Engine paraît techniquement viable **sans réécriture fondamentale du moteur ni des jeux**.

Le cœur de GPE est déjà bien placé pour cette cible : `Game`, `Frame`, `Framebuffer`, `Input`, `ControlMap`, `VirtualPad`, les primitives CPU et le renderer `wgpu` n'ont pas de dépendance métier à UIKit. `winit 0.30.13` possède un backend iOS/UIKit explicite et recommande justement de créer la fenêtre depuis `ApplicationHandler::resumed`, ce que le runtime GPE fait déjà. `wgpu 30.0.0` active Metal sur macOS **et iOS**.

Le principal défaut architectural actuel est confirmé : plusieurs chemins assimilent encore :

```text
not(wasm32) == desktop native
```

Cela est déjà identifié pour Android dans [`ROADMAP.md`](../../ROADMAP.md), et iOS renforce la nécessité de supprimer cette hypothèse aux frontières plateforme. Il ne faut cependant **pas** convertir GPE en framework multi-backends générique avant d'avoir un consommateur. Quelques `cfg` ciblés et de petits backends iOS suffisent pour le premier vertical slice.

Les principaux écarts constatés sont :

1. `gilrs 0.11.2` ne documente pas iOS parmi ses plateformes supportées : le backend gamepad actuel ne doit pas être considéré comme viable sur iOS ;
2. `directories 6.0.0` traite les plateformes autres que Linux/Redox/macOS/Windows selon les conventions Linux : ce n'est pas une stratégie de stockage iOS correcte ;
3. `rodio 0.22.2` n'est **pas** à écarter : son feature `playback` utilise `cpal 0.17`, et `cpal 0.17.3` possède explicitement un chemin iOS/CoreAudio ainsi qu'une dépendance `AVAudioSession` ;
4. le runtime GPE implémente `resumed`, mais **pas `ApplicationHandler::suspended`**. Sur iOS, Winit traduit `applicationWillResignActive` en `Suspended`. Le timing est déjà correctement réinitialisé au `resumed`, mais la suspension, le reset d'input, la politique de redraw et la reprise GPU/audio doivent être explicitement validés ;
5. le packaging iOS n'existe pas. Le chemin upstream recommandé par Winit est une bibliothèque Rust statique appelée depuis un petit projet Xcode/UIKit.

La meilleure prochaine étape n'est donc pas un grand refactor mais un **spike Snake tactile sur simulateur iOS**, avec gamepad, stockage persistant et audio réel temporairement no-op.

---

## Légende de confiance

Cette étude distingue volontairement trois niveaux :

- **CONFIRMED** : constaté dans le dépôt audité ou documenté explicitement par le projet upstream concerné ;
- **LIKELY** : architecture et upstream compatibles, mais GPE n'a pas encore été construit/exécuté sur iOS ;
- **UNKNOWN / REQUIRES SPIKE** : seule une compilation ou une exécution sous Xcode/iOS peut trancher proprement.

Aucun build iOS GPE n'a été exécuté dans cette étude. Les SDK Apple et le simulateur Xcode sont donc la frontière de validation restante.

---

## 1. État actuel du moteur audité

### Versions réellement présentes

Le `Cargo.toml` de `main` fixe actuellement notamment :

```text
Rust edition          2024
rust-version          1.97.1
winit                 0.30.13
wgpu                  30.0.0
rodio                 0.22.2
cpal                  0.17.3 (transitif, Cargo.lock)
gilrs                 0.11.2
directories           6.0.0
```

Le découpage de dépendances actuel est essentiellement :

```toml
wasm32
  -> web-sys / wasm-bindgen / WebAudio / WebStorage

not(wasm32)
  -> directories
  -> gilrs
  -> rodio
```

C'est précisément le point qui doit évoluer avant une première compilation iOS utile.

### Architecture runtime observée

[`src/platform.rs`](../../src/platform.rs) :

- utilise `winit::application::ApplicationHandler` ;
- crée la fenêtre et initialise le renderer dans `resumed()` ;
- reçoit `WindowEvent::Touch` et le convertit dans l'espace framebuffer ;
- borne `Frame::delta_time` à **100 ms** ;
- réinitialise la référence temporelle lors de `resumed()` et des changements de focus ;
- reset clavier/souris/touches sur perte de focus ;
- demande un redraw depuis `about_to_wait()` ;
- ne surcharge actuellement **pas** `ApplicationHandler::suspended()`.

[`src/renderer.rs`](../../src/renderer.rs) :

- crée une `wgpu::Instance` générique ;
- crée la `wgpu::Surface` depuis la fenêtre Winit ;
- choisit un adapter compatible avec cette surface ;
- n'exige aucune feature GPU particulière ;
- upload le framebuffer CPU dans une texture `Rgba8UnormSrgb` ;
- rend un triangle plein écran avec échantillonnage nearest ;
- gère `Lost` / `Outdated` en demandant une reconfiguration de surface.

[`src/input.rs`](../../src/input.rs) :

- possède déjà un type `Touch` indépendant de Winit ;
- possède l'abstraction logique gamepad (`GamepadId`, `GamepadButton`, profils, connexions) ;
- `reset_window_devices()` efface clavier, souris et événements touch.

[`src/audio.rs`](../../src/audio.rs) et [`src/storage.rs`](../../src/storage.rs) exposent déjà des traits stables (`Audio`, `LocalStorage`) avec implémentations de plateforme séparées. La bonne direction iOS est donc de **conserver ces contrats**, pas d'exposer UIKit/CoreAudio/Foundation aux jeux.

### Snake est déjà un bon consommateur de spike

Le cœur Snake possède explicitement `SnakeInteractionMode::Touch`, un `VirtualPad` et un layout tactile. L'entrypoint Web utilise déjà cette variante dans [`examples/snake_web.rs`](../../examples/snake_web.rs). Le spike iOS peut donc reprendre ce même choix sans inventer un contrôle mobile spécifique.

---

## 2. Matrice de faisabilité

| Domaine | État GPE actuel | Évaluation iOS | Travail estimé | Confiance |
| --- | --- | --- | --- | --- |
| `Game` / `Frame` / gameplay | Rust portable | inchangé attendu | faible | **CONFIRMED** côté GPE |
| framebuffer CPU | mémoire Rust + pixels RGBA | portable | faible | **CONFIRMED** |
| `wgpu 30` | surface générique + GPU | Metal supporté upstream | faible à moyen | **CONFIRMED upstream / LIKELY GPE** |
| `winit 0.30.13` | `ApplicationHandler`, fenêtre dans `resumed` | UIKit supporté upstream | moyen | **CONFIRMED upstream / LIKELY GPE** |
| touch | `WindowEvent::Touch` -> `Input::Touch` -> `VirtualPad` | chemin très favorable | faible | **LIKELY** |
| timing | clamp 100 ms + reset au resume/focus | bonne base | faible | **CONFIRMED** |
| lifecycle | pas de `suspended()` GPE | incomplet | moyen | **CONFIRMED gap** |
| gamepad | `gilrs` sur tout `not(wasm32)` | backend iOS non documenté | moyen | **CONFIRMED gap** |
| audio | Rodio -> CPAL | CoreAudio/iOS existe | moyen | **LIKELY**, lifecycle à tester |
| storage | `directories` + filesystem | conventions non adaptées iOS | faible à moyen | **CONFIRMED gap** |
| packaging | archives Win/Linux | projet Xcode absent | moyen | **CONFIRMED gap** |
| OBS mirror | compilé pour tout `not(wasm32)` | pas nécessaire au mobile | faible | **REVIEW LATER** |

---

## 3. Rust et targets iOS

**CONFIRMED.** Rust publie des targets iOS Tier 2 avec `std`, notamment :

```text
aarch64-apple-ios       appareil ARM64
aarch64-apple-ios-sim   simulateur ARM64
x86_64-apple-ios        simulateur x86_64
```

Les targets nécessitent les SDK iPhoneOS/iPhoneSimulator fournis par Xcode. La documentation Rust indique également que les targets ARM64 nécessitent Xcode 12 ou plus récent.

Le target de spike à privilégier sur un Mac Apple Silicon est :

```text
aarch64-apple-ios-sim
```

puis :

```text
aarch64-apple-ios
```

pour le device réel.

Le minimum iOS du target Rust ne doit pas être confondu avec le minimum réel du produit : Winit, WGPU, CPAL, Xcode et la politique de GPE peuvent imposer une version plus récente. Aucun deployment target GPE ne doit être figé avant le spike.

Source : [Rust platform support — Apple iOS](https://doc.rust-lang.org/rustc/platform-support/apple-ios.html).

---

## 4. Winit / UIKit

### Ce qui est déjà favorable

**CONFIRMED.** `winit 0.30.13` possède un module iOS/UIKit explicite.

Sa documentation indique que UIKit doit avoir effectué son initialisation avant les opérations UI et recommande de créer les fenêtres dans :

```rust
ApplicationHandler::resumed(...)
```

GPE respecte déjà cette règle : `PlatformApp::resumed()` appelle `create_window_and_renderer()` uniquement après le callback de reprise.

C'est un point important : l'architecture actuelle n'a pas besoin d'être renversée pour s'adapter au modèle UIKit.

### Intégration Xcode recommandée par Winit

Winit recommande comme intégration simple :

```text
crate Rust compilée en staticlib
        ↓
fonction C exportée
        ↓
petit projet Xcode
        ↓
UIApplicationMain / UIKit
        ↓
runtime Winit/GPE
```

GPE étant en édition Rust 2024, l'exemple upstream utilisant `#[no_mangle]` doit être adapté à la syntaxe Rust 2024 :

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ...
```

Le wrapper est un détail de packaging/bootstrapping ; il ne doit pas contaminer `Game` ou le code des jeux.

Sources :

- [Winit 0.30.13 — iOS/UIKit](https://docs.rs/winit/0.30.13/winit/platform/ios/index.html)
- [Rust 2024 — unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html)

---

## 5. WGPU / Metal

**CONFIRMED upstream.** `wgpu 30.0.0` active par défaut le backend `metal`, documenté comme disponible sur **macOS et iOS**.

Le renderer GPE est particulièrement favorable à ce port :

```text
Framebuffer CPU RGBA
        ↓
queue.write_texture
        ↓
texture wgpu
        ↓
render pass minimal
        ↓
Surface Winit
        ↓
Metal / iOS
```

Il n'utilise actuellement ni extension desktop spécifique, ni shader spécifique à Vulkan/DX12, ni feature GPU exotique.

### Ce qui reste à prouver

**UNKNOWN / REQUIRES SPIKE.** Il faut encore vérifier dans une vraie application iOS :

- création de `wgpu::Surface` depuis la fenêtre UIKit fournie par Winit ;
- sélection effective d'un adapter Metal ;
- format de surface et rendu sRGB ;
- resize/orientation ;
- comportement de la surface après background/foreground ;
- absence de blocage problématique de `pollster::block_on(Renderer::new(...))` dans le callback de lifecycle.

Rien dans le renderer actuel ne signale un besoin de backend graphique GPE spécifique à iOS. Le risque se situe surtout dans le **lifecycle de la surface**, pas dans le pipeline de pixels.

Source : [WGPU 30 — feature flags/backends](https://docs.rs/wgpu/30.0.0/wgpu/).

---

## 6. Touch et input

### GPE

**CONFIRMED.** Le runtime traite déjà :

```text
WindowEvent::Touch
        ↓
touch_from_winit(...)
        ↓
Input::push_touch
        ↓
Touch { id, phase, position }
        ↓
ControlMap / VirtualPad / Game
```

La conversion vers le framebuffer passe par le viewport commun. Snake possède déjà une version tactile réellement consommée par le Web.

### iOS

**LIKELY.** Winit fournit ses événements de fenêtre sur son backend UIKit ; le chemin GPE n'a pas de dépendance Web dans la normalisation tactile.

Le spike doit néanmoins vérifier :

- coordonnées avec scale factor Retina ;
- orientation choisie ;
- multi-touch ;
- `Cancelled` lors d'une interruption ;
- reprise après changement d'application ;
- absence de contact logique bloqué dans `VirtualPad` après suspension.

Aucune nouvelle abstraction tactile ne paraît justifiée.

---

## 7. Gamepad

### Diagnostic actuel

**CONFIRMED gap.** `src/gamepad.rs` sélectionne `gilrs` pour tout `not(wasm32)`. `gilrs 0.11.2` documente Linux/BSD, Windows, OS X et Wasm ; Android est explicitement non supporté et **iOS n'est pas listé**.

Gilrs possède bien une erreur `NotImplemented` et un contexte dummy pour les plateformes non supportées. Cela ne constitue toutefois pas un backend iOS acceptable et il n'est pas souhaitable de conserver la dépendance desktop dans la cible mobile par simple accident de `cfg`.

### Stratégie recommandée

Pour le premier spike :

```text
iOS GamepadInputBackend = no-op
```

Le tactile suffit à Snake et permet de tuer le risque Winit/WGPU sans mélanger un second problème.

Ensuite, si un jeu iOS a réellement besoin d'une manette :

```text
Apple GameController / GCController
        ↓
adaptateur iOS GPE
        ↓
Input::connect_gamepad / set_gamepad_button / profiles
        ↓
ControlMap
        ↓
Game
```

Le framework Apple Game Controller gère les contrôleurs physiques/virtuels, la découverte, les événements de connexion/déconnexion et les profils. Le bon contrat GPE à préserver est donc **`Input`**, pas `gilrs`.

Il n'est pas nécessaire d'introduire dès maintenant un trait public `GamepadBackend`. Un module privé sélectionné par `cfg` ou un type privé de même rôle suffit tant qu'il n'y a pas de besoin plus général.

Sources :

- [Gilrs 0.11.2 — supported features](https://docs.rs/crate/gilrs/0.11.2)
- [Gilrs Error::NotImplemented](https://docs.rs/gilrs/0.11.2/gilrs/enum.Error.html)
- [Apple Game Controller](https://developer.apple.com/documentation/gamecontroller)
- [Apple GCController](https://developer.apple.com/documentation/gamecontroller/gccontroller)

---

## 8. Audio

### Correction par rapport à l'hypothèse initiale

L'étude initiale était trop prudente si elle laissait entendre que Rodio/CPAL était probablement non viable sur iOS.

**CONFIRMED upstream :**

- `rodio 0.22.2` active `cpal 0.17` avec son feature `playback` ;
- le lock GPE résout actuellement `cpal 0.17.3` ;
- CPAL 0.17.3 inclut `aarch64-apple-ios` dans ses cibles docs ;
- CPAL possède des dépendances Apple CoreAudio et, spécifiquement sous `target_os = "ios"`, `objc2-avf-audio` avec la feature `AVAudioSession`.

La conclusion correcte est donc :

> **Le backend Rodio/CPAL actuel est un candidat crédible pour l'audio iOS de base. Il doit être essayé avant de construire un backend audio entièrement spécifique.**

### Ce qui n'est pas résolu pour autant

**UNKNOWN / REQUIRES SPIKE.** Un flux PCM qui sort n'est pas un backend iOS de production complet.

Apple associe l'audio applicatif à `AVAudioSession`. Le comportement de la session contrôle notamment :

- interaction avec le switch silencieux ;
- coexistence avec l'audio d'autres applications ;
- interruptions ;
- background/foreground ;
- changements de route (casque, Bluetooth, etc.).

Apple indique que la session audio d'une app est désactivée lors de certaines suspensions et publie des notifications d'interruption/route. Le backend GPE actuel ne modélise pas encore cette politique : `PlatformAudio::activate()` est vide côté natif.

### Ordre recommandé

1. **Spike graphique/touch : `NoopAudio` sur iOS.**
2. Essayer le `NativeAudio` Rodio/CPAL actuel dans une app iOS réelle.
3. Si lecture one-shot et boucle fonctionnent, conserver Rodio/CPAL.
4. Ajouter seulement la mince couche Apple nécessaire à la politique `AVAudioSession` et au lifecycle.
5. Remplacer Rodio/CPAL uniquement si un problème concret et reproductible le justifie.

Sources :

- [Rodio 0.22.2 — Cargo/features](https://docs.rs/crate/rodio/0.22.2/source/Cargo.toml.orig)
- [CPAL 0.17.3 — Cargo targets](https://docs.rs/crate/cpal/0.17.3/source/Cargo.toml)
- [Apple AVAudioSession](https://developer.apple.com/documentation/avfaudio/avaudiosession)
- [Apple audio interruption notification](https://developer.apple.com/documentation/avfaudio/avaudiosession/interruptionnotification)
- [Apple route-change notification](https://developer.apple.com/documentation/avfaudio/avaudiosession/routechangenotification)

---

## 9. Storage

### État actuel

**CONFIRMED gap.** `platform_storage()` sélectionne actuellement `FileLocalStorage` pour tout `not(wasm32)`. Ce backend utilise `directories::ProjectDirs::data_local_dir()`.

`directories 6.0.0` documente un vrai comportement spécifique pour Linux/Redox, Windows et macOS ; les « autres plateformes » utilisent les conventions Linux. iOS tomberait donc dans une convention qui n'exprime pas correctement le sandbox et les répertoires Foundation.

Il ne faut pas interpréter cela comme « `directories` ne compile pas sur iOS ». Le problème est surtout **sémantique et de lifecycle de données**, ce qui est plus subtil.

### Backend iOS recommandé

Conserver :

```rust
trait LocalStorage {
    fn get(...);
    fn set(...);
}
```

et implémenter un backend iOS qui place les données internes persistantes dans l'**Application Support directory** de l'app.

Apple précise que ce répertoire se trouve dans le sandbox de l'application et qu'il est prévu pour les fichiers de support non directement exposés à l'utilisateur. Cela correspond bien à l'usage GPE actuel : scores, réglages et petit état local.

`UserDefaults` peut être pertinent pour de très petites préférences, mais l'utiliser comme backend général modifierait implicitement les propriétés du stockage. Un petit backend fichier dans Application Support conserve mieux la sémantique actuelle.

Pour le tout premier spike, `NoopStorage` est suffisant : perdre le best score au redémarrage ne doit pas bloquer la validation GPU/touch.

Sources :

- [directories 6.0.0 — platforms](https://docs.rs/crate/directories/6.0.0)
- [Apple `applicationSupportDirectory`](https://developer.apple.com/documentation/foundation/url/applicationsupportdirectory)
- [Apple — Using the file system effectively](https://developer.apple.com/documentation/foundation/using-the-file-system-effectively)

---

## 10. Lifecycle

C'est le domaine qui mérite le plus d'attention après le premier rendu.

### Ce que GPE fait déjà correctement

**CONFIRMED :**

- le `delta_time` de simulation est borné à 100 ms ;
- `resumed()` réinitialise la référence temporelle ;
- `Focused(false)` réinitialise les périphériques fenêtre ;
- le renderer sait reconfigurer une surface `Lost`/`Outdated`.

Ces décisions réduisent fortement le risque de « rattraper » plusieurs secondes de simulation après un retour foreground.

### Ce qui manque

**CONFIRMED gap :** GPE n'implémente pas `ApplicationHandler::suspended()`.

Winit 0.30.13 mappe sur iOS :

```text
applicationDidBecomeActive   -> Resumed
applicationWillResignActive  -> Suspended
applicationWillTerminate     -> LoopExiting
```

Le premier support iOS devra au minimum déterminer une politique explicite pour :

- arrêter les redraws lorsque l'app n'est pas active ;
- réinitialiser timing et input à la transition ;
- neutraliser les contacts tactiles incomplets ;
- suspendre/désactiver l'audio selon la politique choisie ;
- décider par mesure si la `wgpu::Surface` doit être conservée, reconfigurée ou recréée au retour foreground.

Important : Winit impose explicitement la destruction des surfaces au `Suspended` sur **Android**. Sa documentation iOS ne formule pas la même exigence. Il ne faut donc pas généraliser automatiquement la règle Android à iOS ; le spike doit mesurer le comportement Metal/UIKit réel.

Le `resumed()` actuel retourne immédiatement si `self.window.is_some()`. C'est correct si les objets restent valides ; insuffisant si la reprise iOS exige une recréation de ressources. Ce point est **UNKNOWN / REQUIRES SPIKE**.

Source : [Winit 0.30.13 — ApplicationHandler lifecycle](https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html).

---

## 11. Packaging et bootstrap Xcode

Le packaging natif GPE actuel ne couvre que les archives Windows/Linux de Void Canticle. Il ne doit pas être étendu artificiellement au modèle iOS : un `.app` signé n'est pas une archive desktop.

Le modèle minimal recommandé est :

```text
GPE + Snake
   ↓ cargo build --target aarch64-apple-ios-sim
Rust staticlib
   ↓
petit projet Xcode
   ↓
bootstrap C/Swift/ObjC minimal
   ↓
fonction Rust exportée
   ↓
GPE run(...)
   ↓
winit/UIKit + wgpu/Metal
```

Le projet Xcode porte les responsabilités Apple :

- bundle/app target ;
- SDK et deployment target ;
- simulateur/device ;
- signing ;
- orientation et plist/capabilities ;
- app icon/launch metadata ;
- Game Controller capability lorsqu'elle devient nécessaire.

Le bootstrap doit rester mince. Il ne doit pas devenir un second moteur ou une couche de gameplay Swift.

---

## 12. Problèmes de compilation ou de comportement probables

### P0 — `cfg(not(wasm32))` trop large

**CONFIRMED.** Avant un build iOS propre, les dépendances et modules doivent arrêter d'assimiler tout natif au desktop.

Points concernés aujourd'hui :

- `directories` ;
- `gilrs` ;
- `rodio` ;
- branding/window icon ;
- fullscreen shortcut ;
- capture/OBS mirror ;
- sélection `platform_audio()` / `platform_storage()` / gamepad.

Tous ne sont pas des erreurs de compilation ; plusieurs sont simplement de mauvaises responsabilités de plateforme.

### P1 — Gilrs

**LIKELY unsupported at runtime.** Ne pas le faire entrer dans le spike iOS.

### P2 — Storage `directories`

**LIKELY compiles, semantically wrong.** Remplacer la sélection de backend pour iOS.

### P3 — Audio

**LIKELY compiles**, puisque CPAL possède un target iOS explicite, mais le comportement de session doit être validé sous Xcode/device.

### P4 — Lifecycle

**CONFIRMED incomplete.** `suspended()` absent ; reprise surface/audio/input non testée.

### P5 — code desktop non essentiel

`default_window_icon`, changement de titre FPS, raccourci fullscreen et OBS mirror n'appartiennent pas conceptuellement au premier chemin mobile. Ils peuvent être laissés tranquilles s'ils compilent, puis resserrés par `cfg` uniquement si le spike en démontre le besoin. Ne pas lancer un nettoyage général avant d'avoir un échec concret.

---

## 13. Proposition d'architecture incrémentale

L'arborescence théorique :

```text
platform/
  desktop/
  web/
  android/
  ios/
```

est raisonnable à long terme mais **trop ambitieuse comme première modification**.

GPE possède déjà les abstractions importantes dans les bons domaines : `Input`, `Audio`, `LocalStorage`, `Renderer`, `Game`. Le premier port iOS peut rester beaucoup moins intrusif :

```text
platform.rs
  logique Winit commune
  + quelques branches cfg de lifecycle/bootstrap

gamepad.rs
  desktop -> gilrs
  web     -> Gamepad API existante
  ios     -> no-op, puis GameController

audio.rs
  desktop -> Rodio/CPAL
  web     -> WebAudio
  ios     -> NoopAudio pour I0, puis essai Rodio/CPAL + session Apple

storage.rs
  desktop -> directories/files
  web     -> localStorage
  ios     -> NoopStorage pour I0, puis Application Support/files
```

### `cfg` recommandé

Ne plus utiliser `not(wasm32)` comme synonyme de desktop.

Pour les éléments réellement desktop, préférer une liste explicite des plateformes GPE effectivement supportées, par exemple conceptuellement :

```rust
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
```

et des branches explicites :

```rust
#[cfg(target_arch = "wasm32")]
#[cfg(target_os = "android")]
#[cfg(target_os = "ios")]
```

Ne pas introduire un crate de `cfg_aliases`, un registre de plateformes ou un trait générique supplémentaire tant que ces quelques branches suffisent.

### Relation avec Android

[`ROADMAP.md`](../../ROADMAP.md) contient déjà une cible officielle Android A0–A5 et identifie le même défaut « natif = non-WASM ».

L'étude iOS ne doit pas créer une deuxième théorie concurrente. Android et iOS doivent partager le **principe** de frontières explicites, mais leurs contraintes concrètes restent différentes :

```text
Android : AndroidApp / SurfaceView / Vulkan / lifecycle Activity
Apple   : UIKit / staticlib Xcode / Metal / AVAudioSession / GameController
```

Une abstraction mobile commune ne doit apparaître que lorsqu'un vrai code commun Android+iOS est démontré.

---

## 14. Risques

| Risque | Probabilité | Impact | Réduction |
| --- | --- | --- | --- |
| bootstrap Winit/Xcode ne s'intègre pas comme prévu | faible à moyenne | élevé | spike staticlib minimal |
| surface WGPU/Metal échoue ou reprend mal | faible à moyenne | élevé | Snake + logs adapter/surface + suspend/resume |
| `pollster::block_on` pose problème dans lifecycle iOS | inconnue | moyen/élevé | tester avant de changer le modèle async |
| touch Retina/orientation mal mappé | moyenne | moyen | Snake Touch + viewport debug |
| dépendances desktop polluent la target iOS | élevée sans changement | moyen | `cfg` explicites minimaux |
| Rodio/CPAL fonctionne mais session Apple incorrecte | moyenne | moyen | séparer playback de la politique AVAudioSession |
| stockage écrit au mauvais endroit | élevée avec `directories` actuel | moyen | backend Application Support |
| gamepad non fonctionnel | élevée avec `gilrs` | faible pour I0 | no-op puis GameController |
| refactor plateforme trop large avant preuve | évitable | élevé | vertical slice, pas de framework |
| contraintes signing/Xcode ralentissent l'itération | moyenne | moyen | simulateur d'abord, device très tôt ensuite |

---

## 15. Inconnues à tester explicitement

1. `cargo check/build` de GPE pour `aarch64-apple-ios-sim` après séparation minimale des dépendances desktop.
2. Link d'une staticlib GPE dans un projet Xcode minimal.
3. Appel du runtime GPE depuis UIKit sans deuxième event loop concurrente.
4. Création et présentation d'une surface WGPU Metal.
5. Fonctionnement de `pollster::block_on(Renderer::new)` dans `resumed`.
6. Mapping touch exact sur écran Retina et rotation/orientation retenue.
7. Comportement `Suspended -> Resumed` avec surface existante.
8. Politique de redraw quand l'app devient inactive.
9. Rodio/CPAL one-shot puis loop sur simulateur et device.
10. `AVAudioSession` : silent switch, interruption, casque/Bluetooth, background/foreground.
11. Backend stockage Application Support et persistance après relaunch.
12. GameController avec au moins une manette Bluetooth réelle.

---

## 16. Spike technique recommandé

### I0 — Frontière de compilation iOS

Objectif : rendre la cible iOS compilable **sans prétendre avoir fini les services**.

Changements minimaux autorisés :

- séparer les dépendances desktop de la cible iOS ;
- `GamepadInputBackend` iOS no-op ;
- `NoopAudio` iOS ;
- `NoopStorage` iOS ;
- petit crate/target staticlib de bootstrap Snake ;
- aucun changement de gameplay.

Critère : obtenir la bibliothèque ARM64 simulateur et la linker dans Xcode.

### I1 — Snake tactile sur simulateur

Objectif : tuer le risque technologique principal.

```text
Xcode
  -> UIApplication/UIKit
  -> fonction Rust exportée
  -> GPE run
  -> Winit resumed
  -> WGPU / Metal
  -> framebuffer
  -> WindowEvent::Touch
  -> VirtualPad
  -> SnakeInteractionMode::Touch
```

Critères de succès :

- l'app démarre sans crash ;
- un adapter/surface Metal est créé ;
- le framebuffer est visible avec nearest-neighbor correct ;
- le D-pad tactile répond ;
- Snake est réellement jouable ;
- aucun code de règles Snake ne connaît iOS/UIKit.

Ce succès lèverait le risque architectural principal.

### I2 — Device physique précoce

Avant de construire les services, installer le même spike sur un iPhone/iPad ARM64 réel. Le simulateur ne doit pas être utilisé comme preuve définitive de GPU, input ou audio device.

### I3 — Lifecycle robuste

- implémenter la politique `suspended/resumed` ;
- stopper les redraws inactifs ;
- reset timing/input ;
- tester lock, app switch, interruption et retour ;
- ne recréer la surface que si l'expérience l'exige.

### I4 — Storage

- backend `LocalStorage` Application Support ;
- valider best score Snake après relaunch ;
- tester erreurs de création/écriture.

### I5 — Audio

- essayer d'abord Rodio/CPAL existant ;
- valider one-shot et boucle ;
- intégrer la politique `AVAudioSession` strictement nécessaire ;
- tester interruption et changement de route.

### I6 — Gamepad

- backend GameController ;
- mapper vers les types `Input` existants ;
- test Bluetooth réel ;
- aucune dépendance GameController dans Snake/VC.

### I7 — Packaging/distribution

- projet Xcode reproductible ;
- configuration debug/release ;
- signing ;
- icônes/orientation/capabilities ;
- CI macOS seulement lorsque le workflow local est stabilisé ;
- App Store/TestFlight hors du chemin critique initial.

### I8 — Second consommateur

Porter ensuite **Void Canticle** ou **Smart Boy Hero** sans fork du gameplay. Ce deuxième consommateur est la vraie preuve que l'iOS support appartient au moteur et pas au spike Snake.

---

## 17. Critères d'échec utiles

Un spike qui échoue reste informatif. Il faut distinguer :

### Échec architecture moteur

Signal réellement préoccupant :

- impossible de faire fonctionner le modèle Winit/UIKit sans remplacer la boucle runtime ;
- impossible de créer une surface WGPU Metal avec le modèle de fenêtre choisi ;
- nécessité de modifier les règles ou le rendu métier de Snake pour la plateforme.

### Échec service périphérique

Pas une remise en cause du port :

- Gilrs non fonctionnel ;
- Rodio nécessitant une adaptation de session ;
- `directories` donnant un mauvais chemin ;
- OBS mirror/branding desktop non pertinents.

Ces éléments sont précisément les services que les abstractions GPE existantes permettent de remplacer localement.

---

## 18. Conclusion

### Confirmed

- Rust fournit les targets iOS ARM64/device et ARM64/x86_64 simulator.
- Winit 0.30.13 supporte iOS/UIKit et recommande la création de fenêtre dans `resumed`.
- GPE crée déjà sa fenêtre dans `ApplicationHandler::resumed`.
- WGPU 30 active Metal sur iOS.
- Le renderer GPE est générique et minimal.
- GPE a déjà une abstraction tactile, audio, storage et input logique.
- Le split actuel `wasm32` / `not(wasm32)` est insuffisant.
- Gilrs n'est pas un backend iOS documenté.
- `directories` n'est pas iOS-aware et applique les conventions Linux aux autres plateformes.
- CPAL 0.17.3 possède un backend/dépendances iOS explicites.
- GPE n'implémente pas encore `ApplicationHandler::suspended`.

### Likely

- Le cœur GPE et les jeux peuvent rester inchangés.
- Le chemin renderer actuel peut fonctionner via WGPU/Metal.
- Le touch Winit peut alimenter directement le `VirtualPad` existant.
- Rodio/CPAL peut fournir le premier playback iOS avant toute implémentation audio spécifique.
- Un backend fichier iOS très mince peut préserver exactement `LocalStorage`.

### Unknown / requires spike

- link complet GPE + Winit + WGPU sur `aarch64-apple-ios-sim` ;
- comportement exact de `pollster::block_on` dans le lifecycle UIKit ;
- reprise de surface Metal après suspension ;
- comportement audio Rodio/CPAL sur device réel et politique AVAudioSession ;
- détails de touch/scale/orientation sur device ;
- gamepad Apple réel ;
- packaging/signing reproductible.

**Décision recommandée : GO pour un spike iOS, NO-GO pour un grand refactor plateforme avant ce spike.**

---

## Sources externes vérifiées le 2026-08-20

- Rust — Apple iOS targets: <https://doc.rust-lang.org/rustc/platform-support/apple-ios.html>
- Winit 0.30.13 — iOS/UIKit: <https://docs.rs/winit/0.30.13/winit/platform/ios/index.html>
- Winit 0.30.13 — `ApplicationHandler`: <https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html>
- WGPU 30.0.0: <https://docs.rs/wgpu/30.0.0/wgpu/>
- Gilrs 0.11.2: <https://docs.rs/crate/gilrs/0.11.2>
- CPAL 0.17.3 Cargo targets: <https://docs.rs/crate/cpal/0.17.3/source/Cargo.toml>
- Rodio 0.22.2 Cargo/features: <https://docs.rs/crate/rodio/0.22.2/source/Cargo.toml.orig>
- directories 6.0.0: <https://docs.rs/crate/directories/6.0.0>
- Apple AVAudioSession: <https://developer.apple.com/documentation/avfaudio/avaudiosession>
- Apple audio interruptions: <https://developer.apple.com/documentation/avfaudio/avaudiosession/interruptionnotification>
- Apple audio route changes: <https://developer.apple.com/documentation/avfaudio/avaudiosession/routechangenotification>
- Apple Game Controller: <https://developer.apple.com/documentation/gamecontroller>
- Apple GCController: <https://developer.apple.com/documentation/gamecontroller/gccontroller>
- Apple Application Support directory: <https://developer.apple.com/documentation/foundation/url/applicationsupportdirectory>
- Rust 2024 unsafe attributes: <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html>
