# GPE Diagnostics & Runtime Observability — Adversarial Architecture Review

Version de revue : v2.1 (freeze candidate)  
Révision inspectée : `017c698c48bb4a860a0ecac88ce656d0eda861e4` (`feature/audio2-core`)  
Date : 2026-08-27  
Portée : architecture et stratégie d'essai uniquement ; aucune implémentation ni injection de faute exécutée.

## Convention de preuve

- **OBSERVED** : établi directement dans le dépôt ou dans la source verrouillée d'une dépendance.
- **INFERRED** : conséquence architecturale raisonnable des faits observés, non démontrée par une expérience.
- **UNKNOWN** : le dépôt et les garanties documentaires inspectées ne permettent pas de conclure.
- **REQUIRES EXPERIMENT** : doit être mesuré ou falsifié dans une mission ultérieure isolée.

Les numéros de ligne sont ceux de la révision inspectée. Les affirmations spécifiques au dépôt citent fichier et symbole.

## 1. Executive Verdict

**REVISE.** L'idée générale — exposer au consommateur des faits moteur déjà capturés — répond au manque révélé par l'incident WGPU, mais `EngineDiagnosticsSnapshot` n'est pas une abstraction honnête ni suffisamment petite pour être gelée.

Trois faits observés invalident le modèle naïf :

1. **OBSERVED** — il n'existe pas un unique GPU logique. `PlatformApp::renderer` et `ToolWindowState::renderer` peuvent coexister, et chaque `Renderer::new` crée une `Instance`, un adapter, un device et une surface indépendants ([`PlatformApp`, `ToolWindowState`](../src/platform.rs#L203), [`Renderer::new`](../src/renderer.rs#L106)).
2. **OBSERVED** — le consommateur ne reçoit que des emprunts temporaires via `Frame<'_>` ; le renderer, la fenêtre et le cycle de vie restent privés dans `PlatformApp`. Un hook de panic consumer-owned ne peut donc atteindre aucun état moteur actuel après ou en dehors de `Game::update` ([`run`](../src/platform.rs#L175), [`Frame`](../src/platform.rs#L117), [`PlatformApp`](../src/platform.rs#L213)).
3. **OBSERVED** — GPE ne conserve ni identité d'adapter, ni backend, ni erreur WGPU précise. `Renderer::render` réduit `Outdated|Lost` à `SurfaceChanged` et `Timeout|Occluded|Validation` à `Skipped`; aucun uncaptured-error handler ni device-lost callback n'est installé ([`Renderer::render`](../src/renderer.rs#L329)).

La recommandation minimale est un **handle explicite, non global, créé avant `run` et détenu aussi par le consommateur**, donnant une `EngineDiagnosticObservation` best-effort. Celle-ci est un assemblage déclaré de faits immuables, de dernières observations horodatées et d'un petit historique partiel ; ce n'est pas un snapshot atomique. L'identité GPU minimale est `(role, incarnation monotone de session)`, les device et surface restant subordonnés tant que leur durée de vie coïncide avec celle du `Renderer`.

Le noyau GPE ne doit installer ni panic hook, ni writer, ni crash reporter, ni watchdog. La perte totale des diagnostics demeure possible et correcte pour plusieurs classes d'incident. La non-interférence ne peut pas être garantie absolument dans le même processus, surtout sous corruption mémoire ou épuisement réel de l'allocator ; elle doit être formulée comme une propriété bornée, testée et fail-open.

## 2. Repository Evidence

| Preuve | Statut | Conséquence |
|---|---|---|
| `wgpu = "30.0.0"`, `winit = "0.30.13"`, Rust minimal `1.97.1` dans [`Cargo.toml`](../Cargo.toml#L1) ; lockfile à WGPU 30.0.0 dans [`Cargo.lock`](../Cargo.lock#L2551) | OBSERVED | Toute analyse WGPU est spécifique à 30.0.0. |
| Toolchain locale de revue : `rustc 1.97.1 (8bab26f4f 2026-07-14)`, cible hôte `x86_64-pc-windows-msvc` | OBSERVED | Le comportement panic documenté ci-dessous correspond à la toolchain déclarée et utilisée pour la revue ; les artefacts consumer peuvent choisir un autre target/profil. |
| Aucune section `[profile.*]` ni option `panic` dans [`Cargo.toml`](../Cargo.toml) | OBSERVED | Les profils du dépôt ne demandent pas `panic="abort"`; l'artefact final peut néanmoins être surchargé par le consumer/build. |
| `build.rs::git_build_id` calcule SHA court + `*` si fichiers suivis modifiés, sinon `UNKNOWN` ([`build.rs`](../build.rs#L1)) | OBSERVED | Une partie de provenance existe au build, mais `GPE_BUILD_ID` n'est référencé nulle part dans `src`; les fichiers non suivis sont ignorés. |
| `Renderer` conserve surface/device/queue/config, mais ni `Instance`, ni `AdapterInfo`, ni adapter ([`Renderer`](../src/renderer.rs#L86)) | OBSERVED | Adapter/backend/driver doivent être capturés pendant `Renderer::new`; ils ne sont pas récupérables depuis l'API GPE actuelle au panic. |
| Tool window native uniquement ; propre framebuffer/input/renderer ([`ToolWindowConfig`, `ToolFrame`](../src/platform.rs#L58), [`ToolWindowState`](../src/platform.rs#L203)) | OBSERVED | Provenance et incarnation sont nécessaires ; `NOT_APPLICABLE` sur WASM. |
| `sync_tool_window` détruit l'ancien état via `self.tool_window = None`, puis recrée fenêtre et renderer ([`sync_tool_window`](../src/platform.rs#L371)) | OBSERVED | Le rôle `tool` est stable, son instance ne l'est pas. |
| `NativeAudio::new` retombe sur `NoopAudio` via `unwrap_or_default` dans [`platform_audio`](../src/audio.rs#L527) | OBSERVED | L'absence de sink natif est aujourd'hui silencieusement confondue avec un backend noop sans provenance publique. |
| Erreurs CPAL : callback `report_stream_error`, `AtomicBool` pour le premier xrun, puis `eprintln!` ([`audio::native`](../src/audio.rs#L620)) | OBSERVED | Producteur audio backend asynchrone possible ; le contexte de thread n'est pas exposé par GPE. |
| Logs GPE ad hoc seulement : audio, gamepad, OBS mirror ; pas de façade de logging/panic/backtrace ([`gamepad::default`](../src/gamepad.rs#L20), [`CaptureMirror::from_env`](../src/capture_mirror.rs#L54), [`report_stream_error`](../src/audio.rs#L638)) | OBSERVED | Il n'existe pas de système diagnostic transversal à étendre. |
| `ObsMirrorGame` publie par `try_lock` et abandonne en contention, mais le serveur clone les frames et crée un thread par client ([`CaptureMirror::publish`](../src/capture_mirror.rs#L108), [`serve`](../src/capture_mirror.rs#L123)) | OBSERVED | Précédent fail-open local utile, mais pas un modèle de crash diagnostics ni de boundedness agrégée. |
| Aucun code `unsafe`, handler de signal, watchdog ou crash reporter dans `src` | OBSERVED | Les crashs natifs graves sont hors couverture actuelle. |
| Void Canticle n'est pas présent dans ce dépôt ; aucune occurrence de son nom ni de son mécanisme diagnostic | OBSERVED | VC ne peut être audité ici ; son mécanisme décrit par la mission reste une entrée externe, pas une preuve repository. |

## 3. Current GPE Architecture

```text
consumer
  └─ run(config, game) ───────────────────────────────┐
                                                      ▼
                                              PlatformApp<G>
                  ┌───────────────────────────────────┼──────────────────┐
                  ▼                                   ▼                  ▼
        primary Window + Renderer               PlatformAudio       Game value
        + Framebuffer + Input                    + Storage               │
                  │                                                     │
                  └──── borrowed Frame<'_> ── Game::update ◀────────────┘

native optional:
        ToolWindowState
          └─ own Window + own Renderer + Framebuffer + Input
             └─ borrowed ToolFrame<'_> ── Game::update_tool_window
```

**OBSERVED** — `run` construit `PlatformApp` sur la pile puis bloque dans `EventLoop::run_app`; le consumer ne reçoit aucun objet runtime ([`run`](../src/platform.rs#L175)). `PlatformApp` possède tous les sous-systèmes ([`PlatformApp`](../src/platform.rs#L213)).

**OBSERVED** — le chemin natif initialise le primary renderer synchroniquement avec `pollster::block_on`; le chemin WASM crée la fenêtre, lance `Renderer::new` par `spawn_local`, puis reçoit `PlatformEvent::RendererReady` ([`create_window_and_renderer`](../src/platform.rs#L510), [`finish_renderer_init`](../src/platform.rs#L577)).

**INFERRED** — il n'y a actuellement qu'un primary et au plus un tool renderer simultanés, mais l'API `Renderer` ne garantit pas ce plafond à long terme. Le modèle public ne doit pas figer « exactement deux ».

## 4. Current Diagnostics / Logging Mechanisms

- Les erreurs d'initialisation contrôlées deviennent `EngineError` et sont rendues après sortie de l'event loop (`pending_error`) : **OBSERVED**.
- Le titre de fenêtre affiche frame time/FPS toutes les 0,5 s ; ces valeurs ne sont pas conservées ([`render_frame`](../src/platform.rs#L293)) : **OBSERVED**.
- Les surface outcomes précis ne sont ni loggés ni historisés : **OBSERVED**.
- WGPU 30 documente que les erreurs non capturées deviennent par défaut des panics, synchrones ou asynchrones selon backend/circonstance ; GPE ne remplace pas ce comportement ([WGPU 30 `Error`](https://docs.rs/wgpu/30.0.0/wgpu/enum.Error.html), [`Renderer::new`](../src/renderer.rs#L106)) : **OBSERVED**.
- Il n'existe aucun hook de panic, capture de backtrace, journal de session, rotation, persistence incident ou format de rapport dans GPE : **OBSERVED**.
- L'OBS mirror est une fonctionnalité de capture framebuffer explicitement activée par environnement, pas une source d'état moteur fiable au panic : **OBSERVED**.

Conclusion : il n'y a pas un « système existant » à généraliser. Le futur chantier introduirait une nouvelle responsabilité transversale ; son périmètre doit donc rester strictement minimal.

## 5. Diagnostic Access Topology / Consumer Reachability

### Topologie actuelle

```text
consumer panic hook
        X  (aucun chemin)
PlatformApp private state
        ├─ renderer
        ├─ audio
        ├─ window/lifecycle
        └─ transient Frame<'_> only during update
```

`frame.diagnostics()` seul échouerait le cas principal : il serait indisponible lors d'un panic dans `renderer.render`, dans un callback WGPU/audio, avant le premier frame et après `Game::update`.

### Topologie minimale recommandée

```text
consumer bootstrap
  ├─ creates EngineDiagnostics handle H
  ├─ installs its own panic hook capturing read-only H clone
  └─ run_with_diagnostics(config, game, H producer endpoint)
                                      │
                           GPE captures bounded facts
                                      │
consumer hook/debug UI ── try_observe(H) ──> owned, partial observation
consumer formatter/writer (optional, outside GPE)
```

**INFERRED** — un handle clonable explicite (probablement `Arc` sous le capot) résout l'accessibilité sans singleton ni faux `'static`. Il doit exister avant l'appel bloquant à `run`, survivre au runtime et exposer alors `lifecycle = ended/partial`, jamais des références backend.

Le nom et l'intégration exacte (`run_with_diagnostics`, builder ou overload futur) sont une décision d'API. Modifier `EngineConfig` directement casserait les struct literals publiques ; ce n'est pas recommandé sans décision de versioning.

## 6. Incident Taxonomy

| Classe | Observation directe GPE | Avant incident | Après incident / frontière |
|---|---|---|---|
| Panic unwind main | Hook consumer voit payload/location ; GPE peut avoir des caches | Oui | Lecture best-effort possible ; unwinding peut détruire l'état. |
| Panic unwind thread secondaire | Hook s'exécute sur ce thread ; process peut continuer si panic joint/ignoré | Oui | Rapport final dépend de la politique consumer, pas de GPE. |
| `panic=abort` | Hook normalement exécuté avant abort | Oui | Pas d'unwind, destructeurs ni flush fiable. |
| WGPU uncaptured error → panic | WGPU 30 default handler panique | Seulement faits déjà capturés | Callback/point de panic et thread non présumables. |
| Device lost | Callback WGPU 30 disponible mais absent aujourd'hui | Adapter/device facts | Événement best-effort ; la livraison finale reste backend-dépendante. |
| Surface failure | `get_current_texture` retourne 6 cas distincts | Config/dernier present | GPE peut capturer précisément au point de match. |
| Audio backend error | CPAL appelle le callback existant | Backend choisi/dernières commandes GPE | État physique device/playback peut être inconnu. |
| `process::abort` | Aucun hook panic | Breadcrumbs antérieurs seulement | **NO FINAL REPORT** correct en-process. |
| Dépendance native / second panic fatal | Variable, souvent aucun chemin Rust fiable | Breadcrumbs antérieurs | Externe requis pour davantage. |
| SIGSEGV/SIGBUS/access violation | Aucun code Rust arbitraire fiable | Breadcrumbs persistés | Async-signal-safe handler spécialisé ou externe ; hors noyau. |
| Allocation failure contrôlée | Injection synthétique possible | Oui | Peut tester la dégradation sans épuiser l'hôte. |
| OOM réel / pression / OOM killer | Allocations et scheduling non fiables | Breadcrumbs préalloués/persistés seulement | Souvent **NO FINAL REPORT**. |
| GPU OOM | `wgpu::Error::OutOfMemory` distinct | GPU facts | Handler peut observer, mais formatter/allocator peut aussi échouer. |
| Freeze | Aucun événement terminal ; process peut encore scheduler certains threads | Heartbeat antérieur | Watchdog interne indicatif ; superviseur externe pour décider/tuer. |
| Deadlock | Aucun événement terminal ; le store diagnostic peut partager le lock bloqué | Heartbeat antérieur | Watchdog interne peut se bloquer à son tour ; externe préférable. |
| Livelock | Activité/heartbeats possibles sans progrès utile | Progress marker antérieur | Un simple heartbeat donne un faux négatif ; externe + oracle de progrès. |
| SIGTERM/TerminateProcess | Platform-specific, aucun handler GPE | Breadcrumbs antérieurs | Rapport final non garanti. |
| SIGKILL | Inobservable par le processus | Breadcrumbs déjà persistés | **NO FINAL REPORT** obligatoire. |
| Startup failure | `EngineError` pour config/window/renderer | Build facts si handle précréé | État subsystem partiel/unknown. |
| Shutdown/Drop failure | Aucun protocole de shutdown explicite ; drops implicites | Phase + last-known | Second panic possible ; lecture dégradée seulement. |

**UNKNOWN** — les comportements exacts de fautes natives de chaque driver, OS et backend audio. **REQUIRES EXPERIMENT** — livraison réelle des callbacks, ordre et contenu par plateforme.

## 7. Coverage Matrix

Légende : `Y` oui en principe, `P` partiel/best-effort, `N` non, `Ext` exige un processus externe. « Final » signifie rapport terminal produit après la faute, pas simples breadcrumbs.

| Incident | Runtime normal | Cache in-process | Hook panic | Watchdog in-process | Processus externe | Final attendu | Breadcrumbs | Garanti ? |
|---|---:|---:|---:|---:|---:|---|---|---|
| panic unwind main | Y | Y | Y | N | P | P | Y | Non : hook/IO peuvent échouer |
| panic + `panic=abort` | Y | Y | Y | N | P | P minimal | Y | Hook oui selon Rust normal ; sortie non |
| `process::abort` | Y | Y | N | N | Y | **NO FINAL REPORT** in-process | P | Non |
| panic secondaire | Y | Y | Y | P | P | Politique consumer | Y | Non |
| panic réentrant/second | P | P | P puis abort | N | Y | **NO FINAL REPORT** fiable | P | Non |
| SIGSEGV/access violation | P | P | N | N/P | Y | **NO FINAL REPORT** noyau | P | Non |
| deadlock | Jusqu'au blocage | Y | N | P (peut partager le lock) | Y | Externe seulement | P | Non |
| freeze/livelock | Jusqu'au gel | Y | N | P | Y | Externe seulement | P | Non |
| kill externe gracieux | Jusqu'au signal | Y | N | N | Y | Non garanti | P | Non |
| SIGKILL/TerminateProcess forcé | Jusqu'au kill | Y | N | N | Y | **NO FINAL REPORT** | P persistés | Non |
| allocation CPU contrôlée | Y | P | P si panic | N | Y | P | Y | Non |
| OOM réel / pression mémoire | P | P | N/P | N | Y | **NO FINAL REPORT** généralement | P | Non |
| OOM killer OS | Jusqu'au kill | P | N | N | Y | **NO FINAL REPORT** | P persistés | Non |
| WGPU OOM | Y | Y | Y si default panic | N | P | P | Y | Non, callback/backend |
| device lost | Y via callback futur | Y | N sauf panic associé | N | P | Pas nécessairement | Y | Non |
| surface lost | Y au match | Y | N | N | N | Non nécessaire | Y | Observation de GPE oui |
| audio backend failure | Y callback/Result | Y | N sauf panic | N | P | Non nécessaire | Y | Non, backend |
| startup failure | P | P | Y si panic | N | P | `EngineError` ou hook | P | Seulement erreurs contrôlées |
| shutdown failure | P | P | Y si panic | N | P | P | Y | Non |

Cas où l'absence de rapport final est techniquement correcte : `process::abort`, SIGKILL/TerminateProcess forcé, crash natif grave, OOM killer, épuisement réel, panic réentrant fatal, freeze/deadlock sans superviseur et toute écriture indisponible. Le design qui promet davantage en-process est rejeté.

## 8. Proposed Model Under Review

`EngineDiagnosticsSnapshot` est rejeté pour trois promesses implicites fausses : « engine » suggère une entité GPU unique, « snapshot » suggère cohérence atomique, et un objet riche suggère que toutes les sections ont été collectées.

Le modèle conceptuel acceptable est :

```text
EngineDiagnosticObservation
  report_status: complete-attempt | degraded | interrupted
  build_facts: immutable
  lifecycle: Observed<Phase>
  renderers: bounded list<RendererObservation>
  audio: Observed<AudioSummary>
  runtime: independent Observed fields
  events: partial bounded history + loss metadata
```

Chaque `Observed<T>` porte au minimum `availability`, `observation_stamp` et `source`. Les noms publics restent à décider ; la sémantique, elle, ne doit pas masquer l'hétérogénéité.

## 9. CAPTURE → READ/MATERIALIZE → PRESENT Separation

| Phase | Responsable | Autorisé | Interdit |
|---|---|---|---|
| CAPTURE | GPE aux mutation points/callbacks | scalaires, enums, petits textes bornés, try-write/drop, stamps | filesystem, formatage riche, appel consumer arbitraire, requête GPU/audio au panic |
| READ/MATERIALIZE | handle partagé | copies propriétaires, try-lock, sections indisponibles, budget fixe | attente non bornée, références backend, fabrication de cohérence |
| PRESENT | consumer | texte/JSON/UI, politique privacy, fichier/upload, footer | être requis pour le succès de CAPTURE, réentrer dans GPE |

Le formatter ne doit produire aucun événement diagnostic. Le writer ne doit jamais être appelé depuis un callback backend. Un échec PRESENT ne doit rétroagir ni sur CAPTURE ni sur le moteur.

## 10. Snapshot Consistency / Freshness / Provenance

Une observation est un ensemble de registres last-known, pas une transaction. Une lecture peut voir GPU à `F38_012`, audio à un temps monotone entre frames, surface à `F38_020`, puis le panic à `F38_021`.

Sémantique minimale par champ :

| Qualité | Sens |
|---|---|
| `KNOWN` | valeur observée, avec source et stamp ; pas nécessairement actuelle |
| `UNKNOWN` | aucune preuve positive ou négative disponible |
| `UNAVAILABLE` | tentative de lecture impossible (contention, section non initialisée, feature absente) |
| `STALE` | valeur utilisable mais antérieure à un seuil/passage de lifecycle explicite |
| `NOT_APPLICABLE` | subsystem volontairement absent pour cette cible/configuration |

`device_lost = false` est interdit tant qu'une source autoritative n'a pas établi cette négation. L'absence d'événement `DeviceLost` signifie seulement « non observé dans l'historique conservé ».

Stamps recommandés : phase, renderer `(role, incarnation)`, frame si l'événement appartient à une frame, temps monotone depuis bootstrap si disponible, producer id + séquence locale. Le wall clock n'est pas nécessaire dans le noyau et peut révéler des données ; le consumer peut l'ajouter.

## 11. Authoritative State vs Diagnostic Mirror

| Donnée | Source autoritative | Représentation diagnostic | Point unique de capture | Drift / défense |
|---|---|---|---|---|
| adapter/backend/device type | `Adapter::get_info` pendant `Renderer::new` | fait historique immuable | immédiatement après adapter choisi | Capture dans le constructeur avant oubli de l'adapter ; init partielle sinon. |
| surface format/modes initiaux | `surface_caps` + `config` | fait/config last-observed | construction/configuration | Construire config + record dans une même fonction ; test contractuel. |
| dimensions | `Renderer::config` | last-observed | dans `Renderer::resize` après succès de configure | Ne jamais écrire depuis `WindowEvent` séparément ; mutation + capture colocalisées. |
| surface error | résultat de `get_current_texture` | fait historique | dans le match de `Renderer::render` | Ne pas reconstruire depuis `RenderOutcome`, qui perd la variante. |
| dernier present | retour de `queue.present`/fin de méthode | last-observed | après `present` | « attempted/published », pas preuve de visibilité physique écran. |
| renderer lifecycle | ownership `Option<Renderer>` / `ToolWindowState` | fait historique | création/remplacement/drop explicite | API de lifecycle commune ; test create-close-recreate. |
| device lost | callback WGPU | last-observed event | callback minimal | `UNKNOWN` avant callback ; callback non garanti comme état exhaustif. |
| audio backend | résultat `NativeAudio::new` / WebAudio context | fait de sélection | init/fallback | Remplacer l'actuel fallback opaque par capture atomique du choix, pas query ultérieure. |
| playback actif | collections `NativeAudio` / `WebAudio` | résumé dérivé ou commandes observées | mêmes méthodes mutatrices | Préférer compteur dérivé au read normal ; au panic le qualifier last-observed. |
| lifecycle runtime | transitions `PlatformApp` | last-observed | sites de transition | Centraliser la transition ; jamais l'inférer de présence de champs. |

Le mirror drift reste possible. Les défenses sont : colocaliser mutation et capture, dériver au read normal quand sans backend call, tests de mutation exhaustifs, nommer `last_observed_*`, et supprimer tout champ dont la drift ne peut être révélée. Une fonction appelée « synchroniser le cache » séparée est insuffisante.

### Évaluation des champs candidats

Abréviations : `A` authoritative au point de capture, `D` dérivé, `LO` last-observed, `H` fait historique ; `init`, `mut`, `frame`, `cb` indiquent la fréquence/contexte. « Texte » signifie allocation bornée à l'init ou extrait opaque borné.

| Champ candidat | Valeur / source | Kind ; capture/fréquence | Coût et contexte | Fraîcheur, sensibilité, drift / décision |
|---|---|---|---|---|
| adapter name | identification incident ; `Adapter::get_info` | H ; renderer init/une fois | texte, init async/native | opaque et potentiellement sensible ; tronquer ; **MVP opt-in** |
| backend | distingue Vulkan/DX12/Metal/Browser ; adapter info | H ; init/une fois | enum, init | stable pour incarnation ; safe-by-construction après mapping ; **MVP** |
| device type | intégré/discret/CPU ; adapter info | H ; init/une fois | enum, init | parfois Unknown ; **utile** |
| driver / driver info | corrélation driver | H ; init/une fois | texte, init | opaque/sensible, browser peut masquer ; **opt-in** |
| enabled features | reproduire capabilities | H ; device init | bitset, init | API WGPU-coupled et vide aujourd'hui (`Features::empty`) ; exposer seulement flags GPE significatifs |
| relevant limits | expliquer validation/alloc | H ; adapter/device init | structure potentiellement grosse | ne garder qu'un petit sous-ensemble motivé ; **hors MVP** sinon |
| surface format | présentation/couleur ; `SurfaceConfiguration` | A-at-config/LO ; init + reconfigure | enum, mut main | stable jusqu'à config ; capture colocalisée ; **MVP** |
| present mode | latency/stall ; config | A-at-config/LO ; init + reconfigure | enum, mut main | pas current après perte ; **MVP** |
| alpha mode | compositing ; config | A-at-config/LO ; init + reconfigure | enum, mut main | faible valeur desktop ; garder avec config si coût nul |
| width/height | corrèle resize/surface | A-at-config/LO ; resize | 2×u32, window/main | stale si callback manqué ; colocaliser dans `resize`; **MVP** |
| renderer role/source | provenance indispensable | H | petit enum + u64, tous events | non sensible ; **obligatoire** |
| dernier resize | contexte incident | H/LO ; resize | stamp + dimensions | resizes intermédiaires remplaçables ; **utile** |
| dernier frame présenté | activité renderer | LO ; frame (overwrite scalar) | compteur/stamp, render thread | appel present ≠ visibilité physique ; **MVP**, nommé attempted/published |
| dernière erreur surface | incident précis ; acquisition result | H/LO ; erreur seulement | enum + stamp, render | aucune erreur observée ≠ preuve d'absence ; **MVP** |
| device lost | callback WGPU | H/LO ; cb rare | enum + texte borné, thread inconnu | unknown avant observation ; **MVP conditionnel** |
| uncaptured errors | catégorie WGPU | H ; cb rare | enum + extrait borné, thread inconnu | capture peut changer le panic default ; décision humaine préalable |
| audio backend | distingue native/web/noop | H ; init/lazy init | enum + init error bornée | état Web peut évoluer unavailable ; **résumé utile** |
| audio device name | corrélation backend | H/LO ; init/device change | texte opaque/backend | GPE ne l'obtient pas aujourd'hui ; sensible ; **hors MVP** |
| playbacks actifs | contexte sonore | D/LO ; chaque mutation | compteurs, game/main | audible réel inconnu, drift multi-collection ; résumé seulement, pas IDs |
| playbacks terminés récents | consumer cleanup | H ; poll/take | ring additionnel | faible valeur moteur, risque volume ; **hors noyau** |
| erreurs audio récentes | backend health | H ; Result/cb rare | catégorie + stamp | message opaque ; **dernier incident seulement MVP audio** |
| bus utilisés | contexte mix | D/LO ; playback/mix mutation | bitset | faible causalité, game semantics ; **hors MVP** |
| total playbacks | charge approximative | D compteur ; start | saturating u64 | ne prouve pas concurrents/audibles ; **hors MVP initial** |
| playback lifecycle détaillé | BGM/debug game | mirror de maps ; fréquent | potentiellement non borné | trop spécifique et drift-prone ; **consumer-owned** |
| uptime | repère session | D depuis bootstrap ; read | clock read | monotone, pas wall clock ; **utile** |
| frame counter | repère simulation/render | LO ; frame scalar | saturating u64 | tool/primary frames distinctes ; définir compteur avant usage ; **MVP** |
| FPS récent | performance | D fenêtre glissante | calcul/atomics frame | observer effect et peu utile au crash OOM ; **hors MVP** |
| delta / max delta | stall récent | LO/D ; frame | 1–2 scalaires | GPE borne simulation dt mais titre utilise raw dt ; nommer source ; **optionnel** |
| état fenêtre | focus/size/lifecycle | LO ; window events | enums/scalars main | tool/primary provenance ; focus faible valeur ; size via surface config suffit MVP |
| tool window | renderer lifecycle | H/LO ; create/drop | incarnation record | native only ; **obligatoire pour non-ambiguïté**, N/A Web |
| lifecycle phase | interprète toute absence | LO ; transitions rares | enum/stamp | mutation oubliée critique ; centraliser ; **obligatoire** |
| OS / architecture / target | reproduction | H build | constantes | contrôlé, peu sensible ; **MVP build facts** |
| native/WASM / capabilities | interprète N/A | H build | bitset/enum | éviter liste de features arbitraire ; **MVP** |

## 12. Ordering / Causality Semantics

Un compteur atomique global ne crée pas une causalité. Pire, un producteur peut réserver `#104`, être préempté, puis publier après `#105`. Il s'agit alors d'ordre de réservation, pas même nécessairement d'ordre de visibilité.

Garanties honnêtes :

- ordre du programme par producteur lorsque celui-ci publie séquentiellement ;
- ordre de drain par le thread moteur pour les messages callback déjà reçus ;
- frame index seulement pour les faits attachés à une frame GPE ;
- temps monotone = ordre temporel approximatif d'observation, pas occurrence physique ni causalité ;
- aucune causalité inter-thread sans relation happens-before explicitement capturée.

Un événement audio entre deux frames porte `frame = none`, temps/producer sequence. Un callback WGPU asynchrone porte son temps d'observation et sa provenance renderer. Le rapport doit afficher « chronologie partielle ; ordre affiché = ordre de publication/drain » et éviter les mots « juste avant » sans preuve.

## 13. Multi-Renderer / Multi-Surface / Multi-Device Analysis

**OBSERVED** — chaque appel à `Renderer::new` crée `Instance::default`, surface, adapter et device ([`Renderer::new`](../src/renderer.rs#L106)). Le primary est créé dans `create_window_and_renderer`; le tool dans `sync_tool_window` ([`platform.rs`](../src/platform.rs#L510)).

Le diagnostic décrit donc des **renderer instances**, chacune ayant actuellement exactement un device et une surface. Le modèle minimal :

```text
RendererSource { role: Primary | Tool | OtherControlledTag,
                 incarnation: u64 session-monotone }
```

Les événements surface/device portent ce `RendererSource`. Aucun `RendererId` public générique, aucun generational arena, aucun identifiant surface séparé n'est justifié aujourd'hui. Si une future refactorisation permet de remplacer device ou surface sans recréer `Renderer`, cette précondition cesse d'être vraie et impose une incarnation subordonnée.

Les types publics doivent exposer des catégories engine-owned (`BackendKind`, `DeviceKind`, `SurfaceFailureKind`) et garder WGPU dans l'adapter interne. Une variante `Other/Unknown` évite de casser l'API à chaque évolution backend.

## 14. Instance Identity / Lifetime / Reuse

Le rôle n'est pas l'identité. `tool` peut être fermé (`self.tool_window = None`) puis recréé ; un changement de taille framebuffer/config détruit aussi l'ancien state avant recréation. Un historique rôle-only fusionnerait deux incarnations.

Politique minimale : compteur u64 de session, incrémenté une fois par construction réussie ou tentée (la politique doit être fixée), jamais réutilisé. Overflow : saturation + `identity_exhausted=true`; ne jamais wrap silencieusement. Le coût d'un u64 par renderer/event est justifié par l'ambiguïté évitée.

Une instance détruite reste uniquement comme événement/fait historique `Destroyed`; elle ne doit plus figurer dans `active_renderers`. Une observation peut contenir « latest ended incarnation » si utile, explicitement non actuelle. **REQUIRES EXPERIMENT** — vérifier create/close/recreate rapide et callback tardif après drop ; un callback tardif conserve l'ancienne incarnation.

## 15. Lifecycle Analysis

Phases minimales : `bootstrapped`, `initializing`, `running`, `shutting_down`, `ended`, plus un indicateur de transition interrompue. Pas besoin d'un enum public exhaustif des backends.

| Moment | État honnête |
|---|---|
| avant `run` | build facts connus ; runtime/subsystems `UNKNOWN` ou not-started |
| config invalide | `ended` avec startup error ; aucun GPU/audio supposé |
| `PlatformApp::new` | audio/storage/gamepad peuvent déjà être initialisés avant fenêtre ; phase partial |
| `Renderer::new` | adapter peut être connu alors que device/surface config échoue ; conserver étapes distinctes |
| WASM renderer async | fenêtre connue, renderer pending ; lecture possible mais partielle |
| running | fields indépendants, pas globalement atomiques |
| fermeture tool | ancienne incarnation ended, rôle actuellement absent |
| shutdown/drop | phase publiée avant drop si chemin normal ; callback tardif possible |
| panic dans `Drop` | données pré-drop seulement ; aucune finalisation garantie |
| après `run` | handle survit, marque runtime ended et cesse de prétendre être current |

**UNKNOWN** — l'ordre exact des callbacks natifs lors de chaque teardown backend. Il doit être considéré hostile jusqu'à expérience.

## 16. Threat / Failure Model

Adversaires : contention, réentrance, deux panics, callback tardif, allocator indisponible, texte backend énorme, writer cassé, état partiel, miroir oublié, compteur saturé, renderer recréé, consumer malveillant/lent, backend tenant des locks, corruption mémoire, freeze et kill externe.

Ce que GPE peut promettre au plus : opérations de capture bornées par design ; aucun wait sur le hot/callback path ; aucune query backend pendant lecture dégradée ; perte signalée lorsque détectable ; aucune cohérence globale revendiquée. Il ne peut pas promettre de survivre à la corruption mémoire, à l'OOM réel ni à un bug de la primitive de synchronisation elle-même.

## 17. Observer-Effect Analysis

Budgets à décider avant code : octets agrégés fixes par handle, nombre maximum de renderer records et d'événements, bytes maximum par texte, opérations par frame, aucune allocation steady-state par événement fréquent, zéro I/O capture.

Ne pas capturer un événement « frame presented » à chaque frame dans le ring : un compteur/stamp remplaçable suffit. Les événements mérités sont rares : lifecycle, surface failure, device lost, uncaptured WGPU category, audio backend error, diagnostics dropped/saturated (compteur, pas événement récursif).

**REQUIRES EXPERIMENT** — CPU/frame, cache misses, distributions de latence et plateau mémoire. La propriété réaliste n'est pas égalité de timing, mais égalité d'état déterministe et overhead sous budgets acceptés.

## 18. Disabled Semantics

Trois modes utiles suffisent :

1. **compile-time absent** : feature désactivée, aucun store/callback/branche sauf compatibilité API éventuellement stub ; référence A/B la plus proche de « n'existe pas » ;
2. **runtime collection off** : handle/API existent, build facts optionnels, aucun callback installé, aucun événement/atomic par frame ; retourne explicitement `collection_disabled` ;
3. **export off** : relève exclusivement du consumer ; GPE collecte mais rien n'est écrit/uploadé.

« Diagnostics disabled » doit toujours indiquer lequel. Runtime-off ne doit pas réserver un gros ring ni installer les callbacks. **REQUIRES EXPERIMENT** — compiler et mesurer absent vs off ; vérifier comportement moteur identique et API sans panic.

## 19. Callback Execution Context Analysis

| Producteur | Contexte observé/garanti | Risque | Règle capture |
|---|---|---|---|
| main/window/render | `ApplicationHandler` et `render_frame` | réentrance event loop, backend locks pendant appel | petites mutations locales ; aucun consumer callback |
| tool window | même event loop aujourd'hui | teardown/recréation pendant sync | provenance incarnation obligatoire |
| WGPU uncaptured error | handler `Fn(Error)+Send+Sync`; docs disent sync ou async, pas de thread garanti | concurrent, backend context inconnu | classer/copier borné, try-publish, aucun WGPU call/log/fs |
| WGPU device lost | callback `Fn(...)+Send+'static`; thread non documenté | tardif, teardown, une seule invocation API | même règle ; capture old incarnation |
| CPAL stream error | `report_stream_error` fourni au builder | audio backend thread probable, non garanti par GPE | atomic category/counter ou bounded try-publish seulement |
| game update | event-loop thread | consumer peut panic/reentrer | capture engine state avant/après sites choisis, jamais appeler presentation |
| worker/panic thread | n'importe quel thread consumer/dependency | deux readers/panics, locks détenus | try-read, budget fixe, section unavailable |

Une signature `Send` autorise le déplacement ; elle ne garantit ni thread, ni absence de lock, ni non-réentrance. **UNKNOWN** jusqu'à documentation/expérience backend. Les opérations interdites en callback : locks bloquants, backend APIs, filesystem, logging standard, formatage arbitraire, allocation proportionnelle au message, appel consumer, panic.

## 20. Concurrency / Locking Analysis

Choix par défaut : état single-writer pour le thread moteur, petits mailboxes/counters non bloquants pour callbacks, lecture par `try_read`. Un `RwLock` global unique rend toutes les sections indisponibles dès qu'une section est contendue et permet au formatter lent de gêner le moteur ; il est rejeté.

Une combinaison raisonnable à prototyper : immutable boot facts dans `Arc`; sous-sections indépendantes avec copies petites ; un ring borné single-writer drainé par le main thread ; callback paths limités à `try_send` ou atomiques. Si aucun lock n'est disponible, `try_observe` rend la section `UNAVAILABLE`, sans retry.

Lock-free MPMC custom est rejeté : publication de records partiels, memory ordering, ABA et reentrance augmentent le risque. Si les expériences démontrent qu'un bounded channel existant alloue/bloque, réduire les événements callback à des slots atomiques « dernière catégorie + dropped count » avant d'écrire une structure lock-free maison.

Mutex poisoning : ne jamais `unwrap`; récupérer n'est pas automatiquement sûr si l'invariant logique est cassé. En panic read, une section poisonnée devient unavailable. Deux panics concurrents peuvent chacun obtenir des fragments ; aucun ne doit attendre l'autre.

## 21. Allocation / OOM Analysis

Acceptable hors chemin critique : allocation unique du handle/ring au bootstrap, `AdapterInfo` et build strings bornés à l'init, copie propriétaire lors d'une lecture normale. Dangereux : `format!`, clone de messages arbitraires, croissance `Vec/String`, backtrace, JSON et filesystem au callback/hook sous OOM.

Le zéro-allocation absolu n'est pas exigé partout. En capture fréquente/callback, les records doivent être préalloués et de taille fixe ou l'événement doit être abandonné. En panic read, une API matérialisant un `Vec` peut échouer par abort allocator sans retour Rust ; prévoir une lecture dégradée vers un buffer consumer fourni ou accepter explicitement **NO REPORT under allocator failure**. Cette seconde garantie est probablement le meilleur MVP.

## 22. Aggregate Boundedness / Record Size / Counter Policy

Le budget doit borner simultanément :

- maximum de renderer incarnations retenues (actives + dernières terminées) ;
- N événements ;
- taille physique de chaque record ;
- texte arbitraire par champ et total ;
- mailboxes callback et compteurs auxiliaires ;
- travail de lecture/formatage.

Tous les textes backend (adapter, driver, erreur, device audio, panic payload consumer) sont `truncate(length)+truncated=true`; idéalement GPE ne stocke que catégories plus un petit extrait opt-in. Une `String` avec capacité historique énorme doit être reconstruite dans un buffer borné, pas seulement `truncate` après allocation.

Compteurs long-lived (`frame`, producer sequence, drops, playback total) : `saturating_add` + drapeau `saturated`; jamais `+=` (qui panic en debug et wrap en release), jamais reset silencieux. Le `next_playback_id += 1` actuel de `NativeAudio`/`WebAudio` est une politique implicite existante ([`NativeAudio::next_playback`](../src/audio.rs#L688), [`WebAudio::start_loop_on_bus`](../src/audio.rs#L1233)); le diagnostic ne doit pas la copier.

Saturation ring : overwrite-oldest est acceptable pour récence, accompagné de `ever_wrapped`, `overwritten_count` saturant et intervalle de séquences conservé. Si publication impossible par contention : increment best-effort d'un dropped counter ; ne pas générer un événement « dropped » dans le même ring.

## 23. Panic-Safety Analysis

GPE ne doit pas installer de hook global. Rust 1.97.1 documente que le hook est global, s'exécute avant runtime de panic pour unwind et abort, et `set_hook` panic s'il est appelé depuis un thread déjà panicking ([`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)). Le consumer doit composer son hook et décider backtrace/persistence/UX.

Chemin dégradé autorisé dans le hook : garde de réentrance thread-local/atomique consumer-owned, lecture `try_*` sans backend, champs prébornés, writer best-effort préouvert si le consumer le juge utile. Interdits : modifier le hook, attendre un lock, appeler GPU/audio/window, allouer sans borne, logguer vers le pipeline qui a causé le panic, garantir un flush.

Avec `panic=abort`, le hook reste appelé mais il n'y a ni unwind ni destructeurs. Avec `std::process::abort`, aucun hook panic, aucun destructeur ni flush Rust fiable ([`std::process::abort`](https://doc.rust-lang.org/std/process/fn.abort.html)). Un second panic dans le hook/unwind peut abort ; seule la capture antérieure peut survivre.

## 24. Signal-Safety Boundary

Panic-safe n'est pas async-signal-safe. POSIX limite un handler à une liste étroite ; appeler allocator, mutex/RwLock, formatage Rust, `println!`, logging ou filesystem de haut niveau depuis SIGSEGV/SIGBUS peut être indéfini ([POSIX `sigaction`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaction.html), [POSIX async-signal-safe set](https://pubs.opengroup.org/onlinepubs/9799919799/functions/V2_chap02.html)). Windows access violations ont une frontière différente mais le même problème de corruption/réentrance.

Conclusion : aucun handler natif grave dans le noyau GPE. LEVEL 3 optionnel (crash reporter/superviseur externe) peut lire des breadcrumbs persistés ou un dump OS. Un signal handler minimal éventuel appartient à une crate/platform spécialisée auditée, pas à cette abstraction.

## 25. WGPU Exact-Version Analysis

Version effective : `wgpu`, `wgpu-core`, `wgpu-hal`, `wgpu-types` **30.0.0** : **OBSERVED** via lockfile.

- `Device::on_uncaptured_error(Arc<dyn UncapturedErrorHandler>)` existe. Sans handler/scope, WGPU 30 appelle un default handler qui panique ; les erreurs sont `OutOfMemory`, `Validation`, `Internal`. Les docs préviennent sync/async ([WGPU 30 `Device::on_uncaptured_error`](https://docs.rs/wgpu/30.0.0/wgpu/struct.Device.html#method.on_uncaptured_error), [`Error`](https://docs.rs/wgpu/30.0.0/wgpu/enum.Error.html)).
- `Device::set_device_lost_callback(Fn(DeviceLostReason,String)+Send+'static)` existe ; reasons `Unknown|Destroyed` ([WGPU 30 method](https://docs.rs/wgpu/30.0.0/wgpu/struct.Device.html#method.set_device_lost_callback)). Sa signature ne garantit pas le thread.
- `CurrentSurfaceTexture` a `Success`, `Suboptimal`, `Timeout`, `Occluded`, `Outdated`, `Lost`, `Validation` ([WGPU 30 enum](https://docs.rs/wgpu/30.0.0/wgpu/enum.CurrentSurfaceTexture.html)).
- GPE traite `Success|Suboptimal` comme présentés, `Outdated|Lost` comme reconfigure, et `Timeout|Occluded|Validation` comme skip : **OBSERVED**.
- `Renderer::new` peut capturer `adapter.get_info()` naturellement avant `request_device`; aujourd'hui il ne le fait pas : **OBSERVED/INFERRED**.

Installer un uncaptured handler change le comportement fatal par défaut ; ce n'est donc pas une instrumentation neutre. La décision « observer seulement » versus « remplacer le panic » est humaine et doit être testée séparément. Pour préserver la sémantique actuelle, un futur handler ne doit pas silencieusement convertir WGPU OOM en continuation non validée.

## 26. Audio Analysis

Native : `PlatformAudio` est `NativeAudio` si ouverture device réussie, sinon `NoopAudio` silencieux. L'état actif est réparti dans plusieurs `HashMap`, `HashSet`, `Vec`; un one-shot clone tous les samples (`sound.samples.to_vec`) et les playbacks/finished lists peuvent croître avec l'usage jusqu'au nettoyage consumer ([`NativeAudio`](../src/audio.rs#L659), [`Audio` API](../src/audio.rs#L66)). Ce sont des sources autoritatives internes, mais les parcourir/coller au panic est risqué.

Le diagnostic minimal audio est : backend selected (`native`, `web`, `noop/fallback`), backend-init outcome, compteurs last-observed de commandes start/stop/fail, dernier backend error catégorisé et stamp. Les titres BGM, chemins de fichiers, intentions musicales et mapping game restent consumer-owned (VC). « Active playbacks » ne doit être exposé que comme dérivé au même mutation point et qualifié last-observed ; GPE ne connaît pas nécessairement ce qui est physiquement audible.

Web : `WebAudio` est lazy, peut devenir `unavailable`, et `activate` reprend le contexte sur interaction ([`WebAudio::context`](../src/audio.rs#L1069), [`PlatformAudio::activate`](../src/audio.rs#L1328)). Les file playbacks ne sont pas implémentés par le backend Web et utilisent les défauts `Audio`. Les callbacks/threads natifs n'existent pas sous la même forme : champs conditionnels, jamais faux placeholders.

## 27. Native vs WASM

Commun : build facts contrôlés, lifecycle logique, renderer role/incarnation, surface outcome WGPU, compteurs/frame stamps, qualité/complétude. Optionnel/unavailable : OS device name, driver info, native signal semantics, filesystem, threads, native tool renderer, CPAL.

WASM crée le renderer async et ignore `tool_window_config`; un panic peut être traduit en console/exception selon glue/browser, sans garantie de fichier ni process exit analogue. Le handle peut être `Rc`/single-thread selon target si l'API abstrait le partage ; imposer `Send+Sync` public partout peut exclure inutilement WebGPU. À l'inverse, le hook natif cross-thread a besoin d'un handle thread-safe. **Open design** : façade sémantique commune avec implémentations cfg, sans prétendre à des garanties identiques.

Adapter/driver strings Web peuvent être absentes, normalisées ou privacy-limited : `UNAVAILABLE/UNKNOWN`, jamais inférées de l'OS navigateur.

## 28. Build Provenance

Champs à coût quasi nul : `CARGO_PKG_VERSION` GPE (`0.1.0` à la révision), `GPE_BUILD_ID`, target architecture/family, debug assertion/profile marker, WGPU version constante générée/maintenue, native/WASM, feature flags significatives. Application version/build doit être fourni et possédé par le consumer.

Limites observées de `git_build_id` : SHA court seulement, dirty ignore untracked, fallback `UNKNOWN`, et la variable n'est pas exposée. **INFERRED** — sans directives Cargo explicites adaptées, la fraîcheur lors d'un changement Git sans source mérite vérification. Ne jamais rapporter `clean`; rapporter `tracked_dirty=false_at_build_script_observation` si l'on garde cette méthode.

Question à six mois : un SHA court dirty peut être insuffisant pour reproduire. Recommandation : exposer les faits disponibles, avec `UNKNOWN`, sans lancer Git au runtime. Le consumer/package pipeline reste responsable de son propre commit, version, assets et flags.

## 29. Sensitive / Arbitrary Text Boundary

Safe-by-construction : enums GPE, dimensions, compteurs, profile category, target family, versions contrôlées, renderer role/incarnation. Opaque/arbitraire : adapter/driver/device names, WGPU/CPAL message, panic payload, path audio, game metadata, hostname/env/user data.

GPE doit annoter ou séparer les champs `opaque_text`, les borner à la capture et les rendre opt-in lorsque leur valeur est faible. Truncation réduit taille, pas sensibilité. Aucun rapport ne doit être marqué « safe to upload ». Le consumer applique consentement, redaction, retention et upload.

## 30. API Surface Review

Surface minimale conceptuelle, sans figer la syntaxe :

```rust
let diagnostics = EngineDiagnostics::new(policy); // before run
install_consumer_hook(diagnostics.reader());
run_with_diagnostics(config, game, diagnostics.writer())?;

let observation = diagnostics.reader().try_observe();
```

Le reader est read-only ; le writer est interne/scellé ou consommé par `run`. `Frame::diagnostics()` peut ultérieurement rendre un reader emprunté pour debug UI, mais n'est qu'un raccourci. Pas d'accès à `wgpu::Device`, `Renderer`, audio mutable ou window.

Éviter un gros enum public d'événements exhaustif si chaque nouvelle variante est breaking. Options : enum `#[non_exhaustive]` de catégories stables + payloads bornés, ou records structurés category/subsystem/source. JSON et format stable sont prématurés ; fournir données Rust structurées et éventuellement un formatter helper non panic-path, versionné comme best-effort.

## 31. Data Ownership & Lifetime

L'observation matérialisée possède ses petites copies ; elle ne retient aucun borrow vers `PlatformApp`. Les boot facts immuables peuvent être partagés. Le handle survit à `run`, mais les champs portent lifecycle/end stamp. Plusieurs runs utilisent plusieurs handles explicites ; aucun process singleton.

Un clone `Arc` est acceptable au bootstrap/callback registration. Il ne faut pas cloner de grandes strings par événement. Pendant teardown, les callbacks gardent au plus un weak/producer endpoint lié à l'incarnation ; si store mort, ils abandonnent. Le consumer ne peut jamais muter les faits moteur ni prolonger la vie d'un `Device`/`Surface`.

## 32. Persistence Boundary

GPE n'écrit aucun crash dump automatiquement. Raisons : chemins/permissions/rotation/privacy/platform relèvent de l'application ; le filesystem peut être readonly, plein ou absent ; WASM n'offre pas la même abstraction ; un write peut bloquer/panic via le formatter.

Le consumer peut écrire un journal préincident et un rapport final best-effort. Pour détecter la troncature : en-tête avec report schema/attempt id, sections numérotées avec statut, puis footer `END REPORT <id>`. Footer absent = sortie interrompue ; fichier syntaxiquement valide sans footer n'est pas complet. Atomic rename/flush ne sont pas garantis au crash et restent politiques consumer.

## 33. Formatting & Human Usability

Sections utiles : INCIDENT (consumer), BUILD, LIFECYCLE/RUNTIME, un bloc par renderer incarnation active/concernée, AUDIO résumé, RECENT EVENTS, QUALITY/LIMITS. Les inconnues et la provenance doivent être sur la même ligne que la valeur. Éviter `Debug` dump et afficher les renderer séparément.

Le moteur peut fournir des labels stables pour ses enums, mais le consumer assemble le rapport. Comparabilité : schema/version, mêmes noms de champs, unités explicites, stamps et footer. Le format riche peut allouer hors chemin critique ; le panic path doit avoir une variante minimale ou accepter son échec.

## 34. Field Quality / Partial Truth Model

Chaque champ critique a : valeur éventuelle, quality, source kind (`authoritative_at_capture`, `derived`, `last_observed`, `historical`), observation stamp et provenance subsystem/renderer. `STALE` est déterminé par règle explicite : changement de lifecycle/incarnation, âge en frames/temps choisi par champ, ou perte de mises à jour détectée. Une valeur stale reste affichée avec son âge ; elle n'est pas remplacée par unknown.

Le modèle évite une explosion de wrappers en groupant les champs qui partagent exactement source/stamp, par exemple `SurfaceConfigObservation`. Ne pas donner une quality unique à tout `RendererObservation` si adapter facts et dimensions n'ont pas la même fraîcheur.

## 35. Whole-Report Completeness / Degradation Model

Le materializer connaît une liste de sections prévues et enregistre pour chacune `attempted/succeeded/unavailable/skipped`. `report_status` est calculé après les tentatives : complete-attempt (toutes tentées, certaines légitimement unknown), degraded (contention/poison/drop), interrupted/truncated (construction ou écriture non terminée).

Un `UNKNOWN` dans une section tentée diffère de `NOT_COLLECTED`. Le writer consumer ajoute attempt id + footer. L'intégrité cryptographique est disproportionnée ; compteur de sections + footer suffit à détecter la plupart des interruptions sans promettre l'atomicité du fichier.

## 36. Fault-Injection Strategy

Aucune faute n'a été exécutée. Toutes les fautes destructives utilisent un binaire enfant piloté par un controller ; artefacts dans un répertoire temporaire dédié ; timeout et kill du seul PID enfant.

### F01–F22 : faits attendus et interdits

| ID | FAULT / PRECONDITIONS | EXPECTED OBSERVABLE FACTS | EXPECTED MISSING | FORBIDDEN EFFECTS |
|---|---|---|---|---|
| F01 | panic main, runtime running | payload consumer, build, phase, primary incarnation, breadcrumbs | causalité/root cause GPU non prouvée | blocage, second panic diagnostic |
| F02 | panic worker après bootstrap | thread/payload, cache partial accessible | frame du thread si aucune association | arrêt moteur imposé par GPE |
| F03 | panic pendant section diagnostic contendue | section `UNAVAILABLE`, autres sections tentées | section lockée | wait/retry infini, poison cascade |
| F04 | formatter/writer injecté en erreur | capture antérieure intacte, writer error | footer/final report possible | récursion, moteur muté |
| F05 | hook/formatter re-panic | garde de réentrance, arrêt selon runtime | rapport final fiable | boucle de hooks, deadlock |
| F06 | build `panic=abort` | hook minimal avant abort, cached facts | destructors/flush | prétendre unwind/complet |
| F07 | `process::abort` enfant | breadcrumbs préécrits seulement | hook/final/footer | attente d'un rapport final |
| F08 | kill externe contrôlé | breadcrumbs antérieurs | hook/final | GPE intercepte magiquement |
| F09 | hang boucle enfant | dernier heartbeat, superviseur timeout | cause exacte | watchdog in-process déclaré fiable |
| F10 | deadlock volontaire enfant | dernier heartbeat, timeout externe | stacks sauf outil externe | blocage du controller |
| F11 | alloc failure synthétique au materialize | capture fixe continue ou read error | texte/owned observation | vraie saturation RAM hôte |
| F12 | validation WGPU synthétique/supportée | renderer provenance + Validation | thread/causalité non garantie | callback backend appelle formatter |
| F13 | événement WGPU OOM simulé | category OOM + incarnation | mémoire GPU réelle | consommation GPU réelle par défaut |
| F14 | device-lost path simulé | reason/message borné, old incarnation | récupération physique garantie | appel WGPU depuis callback |
| F15 | surface Lost puis Outdated injectés | deux catégories distinctes + action GPE | visibilité écran | collapse en `SurfaceChanged` diagnostic |
| F16 | audio error injectée | category/backend/stamp, xrun policy | état physique device | `eprintln` requis au succès |
| F17 | buffer saturé | taille plateau, oldest overwritten, drop/wrap counts | historique complet | croissance mémoire |
| F18 | producers concurrents | records intègres, producer order déclaré | causalité totale | torn record, wait global |
| F19 | échec à chaque étape startup | build + phase + étapes déjà réussies | subsystems futurs | données inventées |
| F20 | teardown failure/callback tardif | ended/teardown phase, ancienne incarnation | current renderer supposé | use-after-free/réutilisation id |
| F21 | permission/disk/write failure simulé | erreur consumer, capture intacte | footer/fichier final | panic engine, retry infini |
| F22 | compile absent puis runtime off | statut disabled exact, jeu identique | runtime events | callbacks/allocations cachés en off |

### Terminaison, oracle, isolation et risque

| ID | TERMINATION EXPECTATION / PASS-FAIL ORACLE | CLEANUP / ISOLATION | HOST RISK | CI |
|---|---|---|---|---|
| F01–F05 | exit/panic conforme au profil ; tous expected présents, forbidden absents, footer selon cas | un subprocess/scénario, tempdir | faible | oui |
| F06 | abnormal exit après hook ; pas de drop/flush attendu | artefact build dédié | faible | oui |
| F07–F08 | abnormal/killed ; **NO FINAL REPORT**, seulement breadcrumbs | PID enfant exact | faible | oui, kill portable adapté |
| F09–F10 | controller timeout puis tue l'enfant ; dernier heartbeat seulement | job/process group, timeout court | faible-moyen | oui si robuste |
| F11 | process suit le mode synthétique, mémoire plateau | allocator/failpoint contrôlé, jamais RAM globale | faible | oui |
| F12–F16 | continuation ou terminaison exactement selon politique déclarée ; provenance correcte | adapter logiciel/mock si possible, skip documenté | moyen | conditionnel backend |
| F17–F18 | hash état jeu égal, mémoire fixe, counters exacts, aucun record partiel | workload déterministe | faible | oui |
| F19–F21 | erreur/exit contrôlé, matrice de phases exacte, aucune panne secondaire | tempdir/read-only fixture, subprocess | faible | oui |
| F22 | A/B outputs égaux ; absent/off coûts conformes | builds séparés | faible | oui |

« Plus de sortie » n'est jamais l'oracle. Une sortie absente est PASS pour F07/F08 si les breadcrumbs et l'absence attendue correspondent au contrat.

### Expériences platform-specific hors CI normale

| FAULT | PRECONDITIONS / ATTENDU | ISOLATION / RISQUE | ORACLE |
|---|---|---|---|
| SIGSEGV/access violation réel | symboles/dump externe optionnel ; aucun rapport GPE exigé | VM/container dédié ; risque moyen | process meurt, GPE ne deadlock/recurse pas |
| épuisement mémoire réel | aucun final garanti | VM/cgroup/job memory limit ; risque élevé | hôte reste sain, contrat `NO FINAL REPORT` |
| device/driver loss réel | breadcrumbs + callback si livré, sinon unknown | machine dédiée, driver reset possible ; élevé | aucune sur-promesse ; provenance si événement |
| OOM killer OS | breadcrumbs persistés seulement | VM/cgroup uniquement ; élevé | kill observé par controller, aucun final requis |

## 37. Fault-Injection Oracles

Oracle commun : comparer faits structurés, qualités, section attempts, loss counters, exit kind et footer à un manifest attendu par scénario. Échec si un forbidden effect survient même si le rapport est riche. Les timeouts font échouer toute faute sauf F09/F10 où ils sont l'événement attendu et doivent être bornés.

Les tests doivent pouvoir affirmer explicitement `UNKNOWN`, `UNAVAILABLE`, `STALE`, `NOT_APPLICABLE`, `NOT_COLLECTED`; une valeur par défaut (`0`, chaîne vide, `false`) est un FAIL si elle masque l'absence.

## 38. Subprocess Isolation Strategy

Le controller lance exactement un enfant avec `--diagnostic-scenario Fxx`, transmet tempdir et budget, observe stdout/stderr/exit/timeout, puis inspecte les artefacts. Les enfants destructifs ne partagent ni test runner, ni fichier, ni port. Sur timeout, le controller valide le PID/job puis tue uniquement ce groupe. Les scénarios crash ne tournent jamais dans le process `cargo test` principal.

OOM réel, SIGSEGV et driver reset restent opt-in, platform-labeled et hors poste développeur par défaut. Aucun test ne consomme toute la RAM hôte.

## 39. A/B Observer-Effect Strategy

Même jeu déterministe et mêmes inputs seedés : A compile-time absent, B runtime off, C enabled, D enabled+saturated, E enabled+consumer never reads, F enabled+writer unavailable. Comparer hash d'état par frame, framebuffer final, nombre de frames, allocations, RSS plateau, CPU/frame percentiles, stalls et volume.

Oracle fonctionnel : `state_hash(A)==...==state_hash(F)` hors sorties diagnostics et timing explicitement toléré. Oracle performance : budgets humains fixés avant mesure, pas après. Répéter native/WASM et primary+tool. **REQUIRES EXPERIMENT** — déterminisme du scheduling fenêtre/GPU ; isoler le hash simulation du present réel.

## 40. Long-Run Stability Strategy

Exécuter millions d'événements, resizes rapides, créations/destructions tool, playbacks, callbacks concurrents, avec reader absent puis lent. Échantillonner RSS/heap après warm-up, allocations cumulées vs live, capacité de chaque collection, CPU par tranche et counters saturants.

PASS : mémoire live atteint un plateau compatible avec le budget ; aucune collection auxiliaire ne croît avec le nombre total ; taille record et texte restent bornés ; overwrite/drop exact ; coût/event stable ; aucun consumer nécessaire. Tester près de l'overflow via seed de compteur, pas par attente réelle de `u64::MAX`.

## 41. Top 5 Architecture-Breaking Scenarios

| Rang | Hypothèse cassée / mécanisme concret | Composant | Gravité / probabilité | Détection actuelle | Modification nécessaire | Statut |
|---:|---|---|---|---|---|---|
| 1 | Le hook consumer peut lire le moteur. En réalité `run` enferme `PlatformApp`; `Frame<'_>` expire avant `renderer.render` et n'existe pas dans callbacks/teardown. | API `run`/ownership | Critique / certaine pour design Frame-only | Aucune | handle explicite pré-`run`, non global, survivant | OBSERVED |
| 2 | « GPU engine » unique. Primary et tool ont instances/devices/surfaces indépendants ; tool est recréé et rôle réutilisé. | renderer/tool lifecycle | Critique / élevée | Aucune identité/événement | role + incarnation monotone ; provenance de chaque fait | OBSERVED |
| 3 | Cache = état actuel. Resize/audio/lifecycle ont plusieurs mutation points ; un update oublié produit un rapport faux sans race. | mirror transversal | Critique / élevée à moyen terme | Aucun contract test | colocaliser, dériver, nommer last-observed, supprimer champs non défendables | INFERRED |
| 4 | Ring borné = mémoire bornée et chronologie vraie. Strings/callback queues/anciennes incarnations peuvent croître ; global sequence ne prouve pas causalité. | history/concurrency | Élevée / élevée si design naïf | Aucune | budgets agrégés + texte borné + ordre partiel explicite | INFERRED |
| 5 | Diagnostic survive au moteur. Panic sous lock/OOM + formatter/writer peut bloquer, allouer ou repanic et transformer un incident en abort/deadlock. | read/present/panic | Critique / moyenne | Aucun hostile test | try-read sections, séparation PRESENT, réentrance guard, `NO REPORT` accepté | REQUIRES EXPERIMENT |

## 42. Rejected Alternatives

- Un `EngineDiagnosticsSnapshot` atomique global : cohérence impossible sans lock/stop-the-world coûteux et toujours fausse vis-à-vis des backends.
- Diagnostics seulement dans `Frame<'_>` : inaccessible au cas de panic principal.
- Singleton global : ambigu pour tests/multiples runs, lifecycle artificiel, composition de hooks difficile.
- Hook panic installé par GPE : ressource globale, conflit consumer/tests/WASM, persistence/UX hors moteur.
- Query GPU/audio/window au panic : deadlock/réentrance/backend défaillant.
- Callback consumer direct : code arbitraire peut bloquer/panic dans contexte backend.
- Ring MPMC lock-free maison : complexité de publication injustifiée avant mesures.
- Logs textuels comme source de vérité : non structurés, non bornés, non attribués aux incarnations.
- Dump de toutes les limites/features WGPU : coût/bruit/API backend sans valeur démontrée.
- Crash reporter/signal handler universel : hors scope et fausse garantie cross-platform.

## 43. Minimal Recommended Architecture

### Niveau 1 — runtime normal

Handle explicite et observations structurées ; build facts ; lifecycle ; primary/tool renderer facts ; surface outcomes distincts ; résumé audio minimal ; compteurs remplaçables. Lecture normale pour debug UI.

### Niveau 2 — incident in-process dégradé

Même handle, cache-first, try-read par section, historique rare borné, loss/truncation metadata, aucune query backend. Consumer hook/formatter/writer. `NO FINAL REPORT` accepté.

### Niveau 3 — externe optionnel

Superviseur/crash reporter seulement pour hang/deadlock/native crash/kill evidence. Hors crate GPE core.

MVP encore plus petit recommandé : build facts + lifecycle + renderer role/incarnation + adapter info initial borné + surface failure/last present + WGPU error category/device lost + handle accessible. Reporter, audio playback detail et événements gameplay restent hors MVP. Si ce noyau ne satisfait pas les budgets/hostile tests, réduire à immutable boot facts + dernier incident par renderer ; même l'historique est alors trop ambitieux.

## 44. Explicit Non-Goals

Pas de télémétrie, OpenTelemetry, profiler, metrics DB, uploader, analytics, logging framework, automatic crash dump, watchdog intégré, debugger, arbitrary game state, BGM semantic model, filesystem policy, symbol server, remote service ni causal tracing.

## 45. Preconditions Before Implementation

1. Décider si remplacer le panic WGPU default est autorisé ou si l'observation doit préserver la terminaison.
2. Fixer l'API de bootstrap/handle sans casser `EngineConfig` et sa stratégie native/WASM.
3. Écrire le contrat de qualité, report completeness, identity et partial ordering avant les structs.
4. Fixer budgets chiffrés : bytes total, records, renderer incarnations, texte, temps capture/read.
5. Lister les mutation points autoritatifs et tests de drift associés.
6. Choisir le sous-ensemble MVP ; exclure audio playback detail et formatter du noyau initial.
7. Spécifier disabled modes et A/B oracle.
8. Préparer seulement ensuite le harness subprocess F01–F22, avec revue sécurité séparée avant exécution.
9. Obtenir/inspecter VC dans une mission dédiée ; ce dépôt ne permet pas de valider sa reachability réelle.

## 46. Open Human Decisions

- Préserver le panic WGPU actuel après capture, ou convertir certaines catégories en résultat contrôlé ? Cette décision change le comportement moteur.
- Accepter une nouvelle fonction/builder explicite ou préparer une évolution majeure de `run` ?
- Quels budgets numériques sont acceptables sur desktop et Web ?
- Adapter/driver/error excerpts sont-ils opt-in par défaut pour privacy ?
- Le MVP inclut-il l'audio backend summary ou seulement GPU/runtime ?
- Le handle doit-il être toujours compilé, feature-gated, ou les deux modes ?
- Combien d'anciennes renderer incarnations conserver : zéro, une par rôle, ou N global ?
- Quel consumer pilote après VC : `tool_window_probe` pour lifecycle/multi-renderer et `snake_web`/`arcade_web` pour WASM ; ces deux familles évitent un design BGM-specific.

## 47. Final Verdict

La proposition n'est pas prête à passer en implémentation sous le nom ni la forme `EngineDiagnosticsSnapshot`. Le besoin transversal est réel, mais le design doit d'abord être réduit à une observation explicitement partielle, accessible par handle pré-runtime, avec provenance par incarnation et coûts agrégés bornés. Les limites de couverture et les `NO FINAL REPORT` doivent faire partie du contrat, non être traités comme échecs ultérieurs.

## Appendix A — Réponses aux 70 questions adversariales

1. Non, pas d'`EngineDiagnosticsSnapshot`; une observation partielle suffit.
2. Build facts + handle + faits renderer par incarnation + derniers surface/WGPU incidents.
3. Panic sous lock/OOM puis read/format bloquant ou allouant.
4. Mirror de resize oublié, ou primary/tool fusionnés, présenté comme current.
5. Rien n'est garanti ; lecture peut échouer totalement et ne doit pas dereferencer les backends.
6. Sections `UNAVAILABLE`, zéro attente.
7. Records déjà alloués seulement ; matérialisation/rapport peuvent manquer correctement.
8. Ne pas le requêter ; utiliser faits de création et callbacks/outcomes déjà observés.
9. Build, adapter/backend, incarnation, config et breadcrumbs rares.
10. Payload/location du panic par consumer, copie best-effort du cache, pas backend state.
11. Cause racine, causalité inter-thread, état physique GPU/audio, rapport final.
12. Unwind, drops, récupération et flush ; le hook normal reste avant abort.
13. Pas de hook ; breadcrumbs antérieurs seulement.
14. LEVEL 3 spécialisé/externe ; pas de Rust arbitraire dans signal handler.
15. Superviseur externe ; watchdog in-process seulement indicatif.
16. Oui seulement avec champs optionnels, renderer async, pas de tool/fs/native callback supposés.
17. Inconnu avant prototype ; budget à fixer et mesurer A/B.
18. Inconnue avant budget ; doit devenir une constante/borne contractuelle.
19. Oui si strings, queues, anciennes incarnations ou collections auxiliaires ne sont pas bornées.
20. Oui via logging/formatter ; interdiction de rétroaction et garde consumer.
21. Oui via locks/backend callbacks ; try-only et séparation de sections.
22. Oui ; toute API diagnostic est faillible, hostile tests obligatoires.
23. Oui, compile-time absent et runtime collection off distincts.
24. Moteur identique ; statut disabled explicite, aucun runtime event/callback caché.
25. Oui si types WGPU/audio fuient ; exposer catégories GPE non exhaustives.
26. Assemblage de last-known values, jamais état atomique global.
27. Qualité par champ + reason/stamp ; pas de sentinel par défaut.
28. `(role, incarnation session-monotone)` sur chaque fait/événement renderer.
29. Build facts seulement puis phase/sections partial/unknown.
30. Phase teardown, anciennes incarnations, callbacks tardifs abandonnés ou attribués à l'ancienne.
31. Window/main, WGPU et CPAL ; seuls types/callback sites sont connus, thread/réentrance backend restent unknown.
32. GPE version + commit/dirty qualifié + target/profile + locked WGPU, application fournie séparément.
33. Tous crash/abort/kill/hang/deadlock/OOM et pannes de writer pouvant tuer le runner.
34. Le diagnostic se dégrade ou disparaît ; il ne modifie pas la politique de terminaison.
35. Abort, SIGKILL, native crash, OOM killer/réel, reentrant fatal, hang sans externe, writer indisponible.
36. Non.
37. Valeur + stamp/âge + label `STALE`; elle reste lisible.
38. Événement historical ended avec incarnation, jamais dans active state.
39. Oui si l'API expose catégories GPE + unknown/other, pas handles backend.
40. Build, adapter/backend, incarnation, surface config/failure, last present, lifecycle, error categories ; le reste doit prouver sa valeur.
41. Toute négation non observée, device/surface physique, causalité, thread callback, état audio audible.
42. Frames normales, resizes répétitifs intermédiaires, événements fréquents redondants ; compter les pertes.
43. Backend calls, blocking locks, fs/logging, arbitrary allocation/formatting, consumer callbacks, panic.
44. Backend calls, waits, unbounded allocation, hook mutation, recursive logger, guaranteed I/O.
45. Presque tout Rust haut niveau : allocator, locks, strings, formatting, stdio/logging/fs abstrait.
46. Overwrite oldest + range/loss counters, ou drop on contention ; jamais croissance/wait.
47. Capture reste bornée ; mémoire plateau ; aucun progrès consumer requis.
48. Aucun fichier garanti ; erreur presentation consumer, cache intact si process continue.
49. Aucun rapport/materialization garanti ; capture préallouée peut seulement tenter de survivre.
50. Qualité dégradée, sections unavailable, aucune sélection arbitraire cachée ; zéro garantie absolue.
51. Handle cloné avant `run`, hors lifetime de `Frame`.
52. Oui conceptuellement : build-only avant, partial pendant, ended après ; à prouver par F19/F20/F02.
53. Non ; handle explicite par run.
54. Sources listées en section 11 ; toute copie est `last_observed`/historical.
55. Colocalisation/derivation/contract tests ; sinon révéler stamp et supprimer la prétention current.
56. Program order par producer et drain/publication défini ; pas de causalité globale.
57. Oui, explicitement.
58. Le rôle oui ; l'incarnation non.
59. Compteur monotone de session.
60. Doit l'être sur les trois axes avant implémentation ; aujourd'hui aucun design chiffré.
61. Buffer fixe/truncation avant allocation proportionnelle + `truncated=true`, opt-in si sensible.
62. Saturation + flag.
63. Quality dans chaque champ/groupe homogène ; completeness séparée au report.
64. Section-attempt map + attempt id + footer/end marker consumer.
65. Oui ; dépendances interdites en section 9.
66. Compile absent, runtime collection off ou export off ; toujours préciser.
67. Compile absent : zéro ; runtime off : API/branche minimale à mesurer, aucun ring/callback souhaité.
68. Adapter/driver/device, backend errors, paths, panic/game metadata, host/env.
69. Enums/versions/counters contrôlés vs opaque text ; consumer redaction/consent avant partage.
70. Les matrices des sections 36–37 spécifient expected, missing, forbidden et oracle pour chaque F01–F22.

## Appendix B — Consumer fit check

- **Void Canticle** : doit garder hook, session journal, backtrace, BGM/game context, persistence et UX. Seuls build/runtime/renderer/surface/WGPU et résumé backend audio générique peuvent migrer. **UNKNOWN** — code VC absent, donc intégration à vérifier.
- **`tool_window_probe`** : invalide l'unicité GPU et teste fermeture/recréation, focus et deux renderer natifs ([`examples/tool_window_probe.rs`](../examples/tool_window_probe.rs)).
- **`snake_web` / `arcade_web`** : invalident toute API native-only et testent init async, absence tool window, WebAudio et absence de filesystem classique ([`examples/snake_web.rs`](../examples/snake_web.rs), [`examples/arcade_web.rs`](../examples/arcade_web.rs)).
- **Smart Boy Hero ISO** : erreurs audio consumer-side et boucles montrent que le contexte sémantique audio doit rester au jeu ([`examples/smart_boy_hero_iso/game.rs`](../examples/smart_boy_hero_iso/game.rs#L1265)).

REVISE
