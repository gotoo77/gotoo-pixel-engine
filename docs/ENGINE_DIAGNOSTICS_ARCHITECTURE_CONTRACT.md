# GPE Diagnostics Architecture Contract

Statut : **FROZEN**  
Version : **1.0**  
Mission : MISSION 1 — DIAGNOSTICS ARCHITECTURE CONTRACT  
Date de freeze : **2026-08-27**  
Portée : contrat architectural documentaire ; aucune implémentation.

## 1. Autorité et vocabulaire

Ce contrat transforme les conclusions de [`ENGINE_DIAGNOSTICS_ADVERSARIAL_REVIEW.md`](ENGINE_DIAGNOSTICS_ADVERSARIAL_REVIEW.md) en règles minimales applicables au futur MVP. En cas d'ambiguïté, la revue reste l'autorité principale ; une contradiction factuelle doit être résolue avant implémentation.

Les statuts employés sont :

- **FROZEN DECISION** : décision proposée comme normative pour le MVP ; elle ne doit plus changer après validation humaine du contrat sans réouverture explicite.
- **PROPOSED / REQUIRES EXPERIMENT** : valeur initiale nécessaire pour borner le design, mais non encore validée par mesure.
- **HUMAN DECISION REQUIRED** : choix qui ne peut pas être déduit honnêtement du dépôt ou des orientations reçues.
- **OUT OF MVP** : exclu de la première implémentation, même si une extension future reste possible.

Les noms de types et fonctions en monospace dans ce document sont conceptuels sauf mention contraire. Ils ne constituent pas encore une API Rust approuvée.

## 2. Résultat contractuel

Les décisions structurantes suivantes sont **FROZEN DECISION** :

1. `EngineDiagnosticsSnapshot` est abandonné.
2. Le résultat d'une lecture est une observation explicitement partielle, composée de faits immuables, d'états `last-observed` et d'événements bornés.
3. L'accès repose sur un handle explicite, non global, créé avant `run()` et détenu par le consumer.
4. GPE capture les faits moteur ; le consumer possède le panic hook, la persistence, le formatage final, l'upload, l'UX et toute supervision externe.
5. CAPTURE, READ/MATERIALIZE et PRESENT sont séparés. PRESENT n'est jamais une condition de succès de CAPTURE.
6. Le comportement fatal WGPU actuel est préservé par défaut. Le MVP n'installe pas `Device::on_uncaptured_error()`.
7. L'identité minimale d'un renderer est son rôle plus une incarnation monotone de session.
8. Aucun champ ne peut être présenté comme `current` s'il n'est qu'une copie ou une dernière observation.
9. L'ordre de l'historique ne constitue pas une preuve de causalité.
10. Toute capture est bornée et non bloquante ; abandonner une donnée est préférable à attendre.
11. LEVEL 1 et LEVEL 2 partagent un unique état diagnostic borné ; LEVEL 2 est seulement son mode d'accès dégradé.
12. LEVEL 3 traverse la frontière du processus et reste hors du noyau GPE et du MVP.
13. Le mode enabled utilise un nouvel entrypoint explicite ; aucun framework builder n'est introduit pour ce chantier.
14. La feature Cargo est `diagnostics`, default-off pour le MVP.
15. L'adapter name est collecté localement par défaut lorsqu'il est disponible, comme texte opaque strictement borné.
16. Les budgets numériques sont des gates provisoires obligatoires, toujours `PROPOSED / REQUIRES EXPERIMENT`.

## 3. Invariants normatifs

Le MVP MUST respecter :

```text
unknown > fabricated certainty
last-observed != current
record order != physical occurrence order != causality
drop > wait
bounded > exhaustive
diagnostic loss > engine interference
```

En particulier :

- une absence d'événement ne prouve jamais l'absence de l'incident correspondant ;
- une valeur par défaut (`false`, `0`, chaîne vide) ne remplace jamais une donnée inconnue ;
- un échec diagnostic ne change pas la décision moteur de continuer, retourner une erreur, panic ou abort ;
- aucune lecture dégradée ne requête activement WGPU, audio, fenêtre, filesystem ou le consumer ;
- aucune capture n'appelle du code de présentation ou un callback consumer arbitraire ;
- aucun budget ne peut être satisfait en laissant croître une collection auxiliaire non comptabilisée.

La non-interférence absolue en cas de corruption mémoire ou d'épuisement réel de l'allocator n'est pas promise. Le contrat promet seulement des opérations diagnostiques bornées et fail-open dans les conditions supportées.

## 4. Contraintes observées du dépôt

Le contrat est fondé sur les contraintes suivantes :

- `run(config, game)` possède `PlatformApp` pendant l'event loop et ne retourne aucun accès runtime au consumer ([`run`](../src/platform.rs#L175), [`PlatformApp`](../src/platform.rs#L213)).
- `Frame<'_>` et `ToolFrame<'_>` sont des emprunts temporaires ; ils ne résolvent pas l'accès depuis un panic hook ([`Frame`](../src/platform.rs#L117), [`ToolFrame`](../src/platform.rs#L78)).
- native GPE peut posséder simultanément un primary renderer et un tool renderer ([`ToolWindowState`](../src/platform.rs#L203)).
- chaque `Renderer::new()` crée sa propre WGPU `Instance`, son adapter, son device et sa surface ([`Renderer::new`](../src/renderer.rs#L106)).
- le tool renderer peut être détruit puis recréé lorsque `sync_tool_window` remplace `ToolWindowState` ([`sync_tool_window`](../src/platform.rs#L371)).
- `Renderer` ne conserve actuellement ni adapter ni `AdapterInfo`; ces faits doivent donc être capturés pendant l'initialisation ou rester inconnus ([`Renderer`](../src/renderer.rs#L86)).
- `Renderer::render` voit les variantes exactes de `CurrentSurfaceTexture`, puis les réduit à trois `RenderOutcome`; la capture précise doit avoir lieu avant cette réduction ([`Renderer::render`](../src/renderer.rs#L329)).
- native audio retombe actuellement sur `NoopAudio` si `NativeAudio::new` échoue, sans exposer l'outcome ([`platform_audio`](../src/audio.rs#L527)).
- WebAudio est initialisé paresseusement et peut devenir indisponible après une tentative ([`WebAudio::context`](../src/audio.rs#L1069)).
- WGPU, WGPU Core et WGPU HAL sont verrouillés en version 30.0.0 ([`Cargo.lock`](../Cargo.lock#L2551)).

Aucune orientation humaine reçue n'est incompatible avec ces faits. L'API actuelle ne peut toutefois satisfaire la topologie demandée sans une future surface d'entrée distincte de `run(config, game)` ou une évolution équivalente. Ce besoin est contractuel ; sa syntaxe exacte reste ouverte.

## 5. Périmètre du MVP

### 5.1 Inclus

- handle explicite et accès hors `Frame<'_>` ;
- mode de collection ;
- provenance du build GPE ;
- lifecycle runtime last-observed ;
- renderer role + incarnation ;
- adapter/backend facts capturés pendant l'initialisation ;
- dernière configuration de surface observée ;
- dernières catégories de failure de surface observées ;
- dernière tentative de present arrivée au point contractuellement défini ;
- device-lost last-observed lorsque le callback documenté le fournit ;
- catégories WGPU observables sans modifier le comportement fatal ;
- historique compact d'événements rares ;
- résumé audio générique strictement limité à backend, initialization outcome et dernière erreur backend générique.

### 5.2 Explicitement hors MVP

- `EngineDiagnosticsSnapshot` ou cohérence atomique globale ;
- panic hook GPE, backtrace, crash writer, filesystem, upload, UX, rotation, retention ;
- watchdog, signal handler, crash dump, supervisor ;
- BGM, playlist, piste, chemin d'asset, intention audio, état musical du jeu ;
- playbacks actifs ou terminés, IDs de playback, bus, volumes, compteurs de sons ;
- FPS, métriques détaillées, profiler, télémétrie ;
- input, gameplay, storage, user data ;
- driver dump exhaustif, limits/features WGPU exhaustives ;
- protocole JSON stable ou format de rapport public gelé ;
- reconstruction d'une chronologie causale totale ;
- harness de crash et exécution de la stratégie F01–F22.

## 6. Topologie d'ownership et d'accès

```text
consumer bootstrap
    │
    ├── creates one diagnostics instance D before run()
    │       └── retains read-only Handle(D)
    │
    ├── may install its own panic hook capturing Handle(D)
    │
    └── transfers/attaches one runtime registration for D to GPE
            │
            ▼
        PlatformApp owns the runtime-side producer authority
            ├── primary renderer capture
            ├── tool renderer capture
            ├── lifecycle capture
            ├── bounded async backend ingress
            └── minimal audio capture

Handle(D) ── try materialize ──> owned partial observation
                                      │
                                      └── consumer-owned PRESENT/persistence
```

### 6.1 Contrat du handle

- Le handle MUST être créé avant le démarrage du runtime.
- Il MUST être clonable pour un hook consumer et d'autres readers autorisés.
- Il MUST être associé à exactement une instance logique de runtime GPE.
- Il MUST NOT être un singleton ou être récupéré par une globale implicite.
- Il MUST NOT exposer de mutation consumer de l'état diagnostic moteur.
- Il MUST NOT contenir ou exposer de référence vers `Renderer`, `Device`, `Surface`, `Window`, `Audio` ou `PlatformApp`.
- Il MAY survivre à `run()` ; dans ce cas le lifecycle last-observed indique `Ended` si cette transition a pu être capturée.
- Plusieurs runtimes simultanés ou successifs MUST utiliser des handles distincts.

### 6.2 Autorité d'écriture

Le consumer ne reçoit aucune autorité d'écriture moteur. La future intégration transmet à GPE une registration/producer authority associée au handle. Cette autorité est consommable par un seul runtime. Une seconde association du même handle MUST être rejetée de manière contrôlée avant démarrage, jamais fusionnée.

### 6.3 Intégration à `run`

**FROZEN DECISION** : ne pas ajouter un champ obligatoire à `EngineConfig`, car les struct literals publiques actuelles seraient cassées. `run(config, game)` reste le chemin historique sans collection runtime. Le mode enabled utilise un nouvel entrypoint explicite, conceptuellement :

```rust
run_with_diagnostics(config, game, diagnostics_registration)
```

Le nom Rust exact MAY être ajusté pour rester cohérent avec le repository, mais la forme architecturale est gelée : nouvel entrypoint, registration explicite, aucun builder général créé pour ce chantier. L'inspection du dépôt n'a trouvé aucun builder GPE canonique ; `DeviceSinkBuilder` est une API interne de dépendance audio et ne constitue pas un précédent d'API moteur.

## 7. Contrat de vérité

### 7.1 Axes indépendants

La qualité d'un champ ne doit pas être représentée par une unique valeur ambiguë. Elle comprend trois axes :

1. **Availability**
   - `KNOWN` : une valeur a réellement été observée ;
   - `UNKNOWN` : aucune valeur honnête n'est disponible ;
   - `UNAVAILABLE` : la lecture a tenté d'accéder à une section mais n'a pas pu le faire ;
   - `NOT_APPLICABLE` : le champ ne s'applique pas à cette cible/configuration.
2. **Freshness**, uniquement lorsqu'une valeur existe
   - `IMMUTABLE_FACT` ;
   - `OBSERVED_AT(stamp)` ;
   - `STALE(observed_at, reason)`.
3. **Representation kind**
   - `AUTHORITATIVE_AT_CAPTURE` ;
   - `DERIVED_AT_CAPTURE` ;
   - `LAST_OBSERVED` ;
   - `HISTORICAL_FACT`.

Ces noms sont conceptuels ; leurs distinctions sont normatives.

### 7.2 Stamps

Un stamp MAY contenir :

- temps monotone depuis la création du handle ;
- frame primaire si l'observation appartient réellement à une frame primaire ;
- frame renderer si un compteur renderer est défini ;
- producer identity et séquence locale ;
- ordre d'insertion dans l'historique.

Un champ absent reste absent ; aucun stamp ne lui est fabriqué. Le wall clock n'est pas requis dans GPE. Un événement audio ou callback backend entre deux frames ne reçoit pas artificiellement le prochain ou le dernier frame index comme temps d'occurrence.

### 7.3 Fraîcheur

- Les build/adapter facts restent des faits historiques immuables de leur incarnation.
- Une configuration de surface est `LAST_OBSERVED`; elle devient explicitement stale après destruction de son renderer, après perte détectée sans nouvelle configuration réussie, ou si une mise à jour manquée est détectée.
- Last-present est toujours `LAST_OBSERVED`; il ne prouve ni visibilité à l'écran ni état actuel du GPU.
- Device lost reste `UNKNOWN` tant qu'aucune observation positive n'a été reçue. « Aucun callback reçu » ne devient jamais `false`.
- Une valeur dont l'âge seul dépasse un seuil n'est stale que si ce seuil est défini pour ce champ. Le MVP n'invente pas un seuil global.

### 7.4 Complétude de lecture

Une observation matérialisée porte, par section, `attempted`, `available` ou la raison d'indisponibilité. Son statut agrégé distingue au minimum :

- toutes les sections prévues ont été tentées ;
- lecture dégradée par contention/section indisponible ;
- collection runtime disabled ;
- collection compile-time absente, observable seulement par le fait que l'API n'est pas construite.

Une observation retournée n'est jamais appelée « complete state ». La détection de troncature d'un fichier ou d'un formatter appartient à PRESENT et au consumer.

## 8. Contrat d'ordre et de causalité

L'historique MUST déclarer son ordre comme ordre d'enregistrement dans le store, pas comme causalité.

Garanties maximales :

- ordre du programme au sein d'un producer séquentiel ;
- séquence locale d'un producer si elle est publiée avec succès ;
- ordre d'insertion/drain dans l'historique ;
- aucune causalité inter-thread sans relation supplémentaire explicitement capturée.

Un compteur global réservé avant écriture ne peut pas être nommé « publication order » si deux writers peuvent publier dans l'ordre inverse. L'implémentation devra soit centraliser l'insertion, soit nommer précisément la séquence `reservation_order`. Le format consumer MUST afficher « partial chronology » ou une formulation équivalente lorsque plusieurs producers sont présents.

## 9. CAPTURE → READ/MATERIALIZE → PRESENT

| Phase | Owner | MUST | MUST NOT |
|---|---|---|---|
| CAPTURE | GPE | utiliser les valeurs déjà disponibles aux mutation points ; borner temps/mémoire ; abandonner en contention ; compter les pertes best-effort | filesystem, formatage riche, consumer callback, attente, backend query additionnelle au panic |
| READ/MATERIALIZE | handle/GPE data layer | `try` uniquement ; produire une copie propriétaire partielle ; libérer toute garde avant retour | attendre la fin d'une capture, requêter backend, garder des borrows runtime, prétendre à l'atomicité globale |
| PRESENT | consumer | appliquer format, privacy, persistence, footer et politique d'échec | rétroagir sur CAPTURE, être appelé depuis un callback backend, être requis pour conserver les faits |

Un panic pendant PRESENT ne produit aucun nouvel événement diagnostic GPE. Un échec writer n'est pas capturé dans le ring moteur, afin d'interdire la récursion diagnostic → logging → diagnostic.

## 10. Modèle de couverture LEVEL 1 / LEVEL 2 / LEVEL 3

### 10.1 LEVEL 1 — NORMAL RUNTIME OBSERVABILITY

Responsabilité principale : **GPE**.

LEVEL 1 utilise le store borné et le handle pendant le fonctionnement normal. Il couvre :

- debug UI et tooling ;
- build provenance ;
- lifecycle ;
- observations renderer et surface ;
- observations WGPU autorisées par le MVP ;
- résumé audio générique ;
- historique rare borné.

L1 utilise la lecture normale du même état diagnostic défini par ce contrat. Il n'autorise ni télémétrie non bornée, ni persistence automatique, ni requête backend supplémentaire lors de la lecture.

### 10.2 LEVEL 2 — DEGRADED IN-PROCESS INCIDENT DIAGNOSTICS

Responsabilité partagée :

- **GPE** maintient les données bornées pré-capturées ;
- le **consumer** possède panic hook, matérialisation finale, formatage et persistence.

L2 MUST réutiliser exactement le même état capturé par L1. Il MUST NOT constituer un second store, un second ring ou un sous-système de crash reporting activé au moment du panic.

Contraintes normatives L2 :

- best-effort et try-only ;
- observations partielles autorisées ;
- aucune query WGPU, audio, window ou filesystem depuis GPE ;
- contention → donnée abandonnée ou `UNAVAILABLE` ;
- aucun wait, retry non borné ou appel consumer depuis GPE ;
- aucune garantie de final report ;
- `NO FINAL REPORT` MAY être le résultat techniquement correct.

Le passage conceptuel de L1 à L2 ne déclenche aucune instrumentation supplémentaire. Il change seulement les contraintes de READ/MATERIALIZE : lecture dégradée, budgets plus stricts et acceptation explicite des sections indisponibles.

### 10.3 LEVEL 3 — EXTERNAL FAILURE VISIBILITY

LEVEL 3 est **OUT OF GPE CORE** et **OUT OF MVP**. Il traverse la frontière d'isolation du processus et peut éventuellement couvrir :

- watchdog ou supervisor externe ;
- hang, freeze ou livelock ;
- deadlock ;
- native crash ;
- SIGSEGV / access violation ;
- evidence de `process::abort`, kill externe ou SIGKILL ;
- crash dump OS ;
- toute classe d'incident que le processus ne peut raisonnablement plus observer lui-même.

L3 MAY exploiter des breadcrumbs ou artefacts produits auparavant par le consumer, mais il ne crée aucune responsabilité supplémentaire pour le core GPE. GPE ne possède ni agent externe, ni dump collector, ni protocole de supervision.

```text
L1 and L2 share the same bounded diagnostic state.

L2 is a degraded access mode, not a second crash-reporting subsystem.

L3 crosses the process/failure-isolation boundary and remains outside GPE core.
```

### 10.4 Matrice de couverture

`Yes` signifie capability normalement visée ; `Partial` signifie last-known/best-effort ; `External` signifie que la visibilité fiable appartient à L3 ; `No` signifie hors responsabilité du niveau.

| Capability / Incident class | L1 | L2 | L3 |
|---|---|---|---|
| debug UI / normal tooling | Yes | Partial si process dégradé | No |
| build provenance / lifecycle last-observed | Yes | Partial | MAY consume prior artifact |
| renderer / surface / last-present observations | Yes | Partial, cache-only | MAY consume prior artifact |
| WGPU observations autorisées par le MVP | Yes | Partial, cache-only | MAY correlate external GPU/driver evidence |
| résumé audio générique | Yes | Partial, cache-only | MAY consume prior artifact |
| panic Rust avec unwind | capture avant incident | try-only partial read par consumer hook | Optional external evidence |
| panic avec `panic=abort` | capture avant incident | hook consumer MAY lire avant abort ; aucun flush garanti | Process exit evidence |
| `std::process::abort()` | breadcrumbs antérieurs seulement | **NO FINAL REPORT** correct | External exit evidence |
| hang / freeze / livelock | last progress seulement | aucun terminal report garanti | External supervisor/watchdog |
| deadlock | last progress seulement | read MAY être contended/unavailable | External supervisor/dump |
| native crash / SIGSEGV / access violation | breadcrumbs antérieurs seulement | **NO FINAL REPORT** noyau | External crash dump/reporting |
| kill externe / SIGKILL | breadcrumbs antérieurs seulement | **NO FINAL REPORT** | External process evidence |
| final human-readable report | No — structured state only | Consumer-owned, non garanti | External tooling MAY produire un artefact |

Cette matrice ne remplace pas la matrice d'incidents détaillée de la revue adversariale ; elle gèle uniquement la frontière de responsabilité architecturale.

## 11. Modes de fonctionnement

### 11.1 Compile-time absent

- L'instrumentation runtime, le store, les callbacks et l'API diagnostics ne sont pas compilés.
- `run(config, game)` conserve son comportement et ne paie aucun coût diagnostic contractuel.
- Le consumer doit compiler conditionnellement son intégration.
- Aucun stub ne doit prétendre que la collection a été tentée.

### 11.2 Runtime collection disabled

- Le support est compilé, mais le runtime est lancé sans registration enabled.
- Aucun ring runtime n'est réservé par GPE, aucun callback diagnostic n'est installé, aucun compteur/frame n'est écrit.
- Si le consumer crée explicitement un handle disabled, celui-ci MAY exposer seulement le mode et les build facts immuables ; il ne contient aucun état runtime.
- L'observation indique `COLLECTION_DISABLED`, jamais `UNKNOWN_RUNTIME` comme si une collecte avait échoué.

### 11.3 Enabled

- Un handle et une registration uniques sont associés au runtime avant son initialisation.
- Le store borné est réservé au bootstrap.
- Les captures MVP et leurs loss counters sont actives.
- Un consumer qui ne lit jamais ne bloque pas GPE et ne provoque aucune croissance.

### 11.4 Export/persistence consumer-side

Ce n'est pas un mode GPE. Le consumer peut exporter ou non une observation dans chacun des modes où un handle existe. Export disabled ne désactive pas la collection. Un échec d'export ne modifie jamais le mode de collection ni l'état moteur.

**FROZEN DECISION** : la feature Cargo du MVP est `diagnostics` et elle est default-off jusqu'à validation expérimentale des coûts et budgets. Une campagne A/B future MAY conduire à une réévaluation explicite ; elle ne peut pas modifier ce défaut silencieusement.

## 12. Identité renderer et durée de vie

### 12.1 Identité minimale

```text
RendererSource = (role, incarnation)
role = PRIMARY | TOOL | OTHER/UNKNOWN future-compatible
incarnation = session-monotone non-zero u64
```

- Une incarnation est allouée au début d'une tentative d'initialisation renderer, avant que tous les faits GPU soient connus.
- Une tentative échouée conserve son incarnation comme fait historique `InitializationFailed` si elle a obtenu un slot diagnostic.
- Une incarnation n'est jamais réutilisée pendant la vie du handle.
- Fermer puis rouvrir le tool renderer produit deux incarnations.
- Device et surface utilisent la provenance de leur renderer tant que leur lifetime reste subordonné à celui-ci, comme dans le dépôt actuel.
- Si une future architecture remplace indépendamment device ou surface dans la même incarnation renderer, le contrat doit être révisé avant cette modification pour introduire une identité/révision subordonnée.

Overflow du compteur : saturation + `IDENTITY_EXHAUSTED`; aucune réutilisation ni wrap. Un renderer peut continuer sans provenance complète si l'identité diagnostic est épuisée.

### 12.2 Rétention

Les records actifs ne sont pas évincés au profit de records terminés. Lorsqu'un nouveau record doit être conservé et que le budget est plein, le plus ancien record terminé est évincé. S'il n'existe aucun slot évictable, GPE abandonne le record diagnostic, incrémente un compteur de perte best-effort et continue le renderer.

Un événement tardif d'une incarnation évincée MAY conserver `(role, incarnation)` sans les anciens adapter facts. Il ne doit jamais être réattribué à l'incarnation courante du même rôle.

## 13. Contrat des données MVP

Chaque ligne ci-dessous est normative. « Indisponible » décrit le résultat diagnostic, jamais une action moteur.

### 13.1 Build provenance

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation / drift | Si indisponible |
|---|---|---|---|---|---|
| version GPE | `CARGO_PKG_VERSION` au build | immutable fact, bootstrap | build GPE ; immuable | aucune mutation runtime | `UNKNOWN_BUILD_VALUE` |
| build id GPE | `GPE_BUILD_ID` produit par [`build.rs`](../build.rs#L1) | immutable fact, bootstrap | SHA court ou `UNKNOWN`; dirty signifie seulement fichiers suivis vus par le script | risque de build-script stale ; aucune commande Git runtime | conserver littéralement `UNKNOWN`, ne pas inférer clean |
| target / arch / native-WASM | cfg/compile-time | enums contrôlés, bootstrap | build GPE ; immuable | aucune | `UNKNOWN/OTHER` forward-compatible |
| build profile / panic strategy | metadata compile-time disponible | immutable fact si prouvable | build de l'artefact final | peut être surchargé par consumer ; ne pas déduire de `debug_assertions` seul si ambigu | `UNKNOWN` |
| WGPU version | dépendance verrouillée/build metadata | immutable controlled text | build GPE | mise à jour dépendance = nouveau build | `UNKNOWN`, jamais version générique |

La version/build de l'application reste consumer-owned et n'entre pas dans le store GPE.

### 13.2 Lifecycle runtime

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| runtime phase | transitions de l'entrypoint/`PlatformApp` | `LAST_OBSERVED` + stamp | handle/runtime unique | handle created, initializing, running, shutting down, ended/startup failed | transition abandonnée → ancienne phase + loss/degraded ; jamais inférer depuis présence de champs |

Phases conceptuelles minimales : `NOT_STARTED`, `INITIALIZING`, `RUNNING`, `SHUTTING_DOWN`, `ENDED`. Startup failure est un outcome de fin, pas la preuve que tous les subsystems sont absents. Aucune transition terminale n'est garantie lors d'abort/kill/crash natif.

### 13.3 Renderer et adapter

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| role/incarnation | site de création PlatformApp/tool | historical identity | runtime handle + role + incarnation | début tentative, success/failure, destruction | slot plein → record perdu ; jamais role-only |
| adapter backend | `AdapterInfo` obtenu pendant `Renderer::new` | immutable fact mappé vers enum GPE | renderer incarnation | après adapter choisi, avant oubli de l'adapter | capture manquée → `UNKNOWN`; aucune query ultérieure |
| adapter device type | `AdapterInfo` | immutable enum GPE | renderer incarnation | même point | unknown/other autorisé |
| adapter name | `AdapterInfo` | opaque text borné + `truncated` | renderer incarnation ; immutable historical | même point | absent/privacy backend → `UNKNOWN`; aucune fabrication |
| renderer lifecycle | ownership et sites create/replace/drop | last-observed/historical transition | renderer incarnation | initializing, ready, failed, destroying, ended | drop implicite non observé → dernière valeur stale/degraded |

Driver et driver info sont OUT OF MVP. Enabled features et limits exhaustives sont OUT OF MVP.

### 13.4 Surface

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| format/present/alpha/size | `Renderer::config` au moment de `surface.configure` | `LAST_OBSERVED_SURFACE_CONFIG` + stamp | renderer incarnation | configuration initiale et chaque `Renderer::resize`/reconfiguration réussie | capture colocalisée obligatoire ; manquée → ancienne valeur, degraded si détecté |
| surface failure | variante exacte de `CurrentSurfaceTexture` | rare historical event + last failure scalar | renderer incarnation | dans `Renderer::render` avant réduction en `RenderOutcome` | contention/saturation → drop counter ; aucune variante reconstruite après réduction |

Les catégories MVP suivent WGPU 30 : `Timeout`, `Occluded`, `Outdated`, `Lost`, `Validation`. `Suboptimal` MAY mettre à jour un scalar last-observed mais ne doit pas nécessairement occuper un événement rare. `Success` n'est pas un événement.

### 13.5 Last present

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| last present | point atteint après `queue.present(frame)` dans `Renderer::render` | scalar `LAST_OBSERVED` + stamp/frame renderer | renderer incarnation | chaque appel arrivé après present | ne signifie pas visible/complete GPU ; si capture abandonnée, ancienne valeur reste avec stamp |

Le nom public ne doit pas être `last_successful_frame_visible`. Il doit refléter seulement le point logiciel réellement atteint.

### 13.6 Device lost

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| device-lost observation | callback WGPU 30 `set_device_lost_callback` | positive historical fact + last-observed reason | renderer incarnation | callback seulement | avant callback : `UNKNOWN`, pas `false`; callback dropped → unknown/degraded |

Le message backend est opaque, tronqué et facultatif. Le reason mappé vers `UNKNOWN_REASON` ou `DESTROYED` ne constitue pas une explication racine. Le callback ne peut appeler aucune autre API WGPU ni PRESENT.

### 13.7 WGPU error category

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| observable WGPU error category | seulement une source qui fournit déjà la catégorie sans changer la politique d'erreur : surface Validation, erreurs contrôlées d'init, futur point explicitement approuvé | historical category, texte opaque facultatif | renderer incarnation + capture source | au point naturel de retour/callback approuvé | uncaptured errors du MVP : `UNKNOWN_NOT_INSTRUMENTED`; aucune analyse de texte du panic pour inventer la catégorie |

`OutOfMemory`, `Validation`, `Internal` ne sont enregistrables via uncaptured handler qu'après réouverture de la décision WGPU de la section 15.

### 13.8 Audio minimal

| Donnée | Source autoritative | Capture / représentation | Provenance et freshness | Mutation points | Drift / indisponibilité |
|---|---|---|---|---|---|
| backend selected | résultat réel de construction/lazy selection | `NATIVE`, `WEB`, `NOOP`, `UNKNOWN` historical/last-observed | runtime audio subsystem | native init/fallback ; Web first context attempt | fallback doit conserver backend=noop + init failure ; capture manquée → unknown |
| initialization outcome | résultat `NativeAudio::new`, lazy `WebAudio::context`, noop selection | `NOT_ATTEMPTED`, `SUCCEEDED`, `FAILED_FALLBACK`, `UNAVAILABLE` + stamp | audio subsystem | chaque tentative significative ; Web peut passer not-attempted → success/failure | jamais inférer success parce qu'un appel play retourne Ok |
| last generic backend error | résultat/callback backend déjà reçu | category + opaque excerpt facultatif, `LAST_OBSERVED` | audio producer + stamp sans frame artificielle | CPAL error callback, Web/native call error approuvé | aucun error reçu ≠ no error ; contention → ancien/unknown + loss |

Tout détail de playback, BGM, fichier, asset et état sémantique est OUT OF MVP. Cette limite est **FROZEN DECISION**.

## 14. Authoritative state et mirror drift

Une donnée dupliquée MUST être capturée dans la même fonction ou le même bloc de contrôle que la mutation autoritative. Les couples obligatoires sont :

- adapter selection → immutable adapter facts ;
- surface configuration mutation → surface observation ;
- surface acquisition outcome → failure observation avant `RenderOutcome` ;
- renderer ownership transition → renderer lifecycle ;
- runtime transition → runtime lifecycle ;
- audio construction/fallback → backend + init outcome.

Une fonction générale appelée ultérieurement pour « synchroniser diagnostics » est interdite pour ces champs. Si la colocalisation est impossible, le champ doit être dérivé sans backend query au read, ou exclu du MVP.

Les tests de mutation/drift sont des préconditions futures d'acceptation de l'implémentation, pas une fault-injection préparée dans cette mission. Ce document ne duplique pas F01–F22.

## 15. WGPU fatal semantics contract

### 15.1 Conflit démontré

WGPU 30 transforme par défaut une erreur non capturée en panic. `Device::on_uncaptured_error()` remplace ce default handler. Il n'existe pas d'API publique pour « observer puis déléguer au handler par défaut ».

Un handler custom qui capture puis appelle `panic!` :

- conserve une terminaison de classe panic dans le cas simple ;
- ne conserve pas nécessairement message, location, thread, timing ni comportement réentrant ;
- peut paniquer dans un contexte callback/lock/backend non garanti ;
- peut aggraver un second panic ou une phase de teardown.

Il ne satisfait donc pas « préserver exactement le comportement fatal actuel ».

### 15.2 Alternatives

| Option | Sémantique | Couverture | Décision |
|---|---|---|---|
| A — aucun uncaptured handler | comportement WGPU actuel inchangé | GPE ne pré-capture pas OOM/Validation/Internal uncaptured ; consumer hook voit éventuellement le panic | **FROZEN pour MVP** |
| B — capture puis panic custom | fatal intentionnel mais comportement exact modifié | meilleure catégorie pré-panic | OUT OF MVP ; exige décision humaine et expériences dédiées |
| C — error scopes ciblés | change quels appels capturent/retournent les erreurs et leur timing | seulement opérations explicitement scoppées | OUT OF MVP sauf besoin renderer séparé |
| D — convertir en récupération | changement fonctionnel majeur | couverture possible mais état moteur à redéfinir | REJECTED par orientation humaine actuelle |

Le MVP MAY installer `set_device_lost_callback`, car il s'agit d'un mécanisme distinct, à condition de ne pas appeler WGPU depuis le callback et de démontrer ultérieurement que l'installation ne modifie pas la politique fatal des uncaptured errors.

## 16. Historique borné

L'historique contient uniquement des événements rares à forte valeur :

- runtime lifecycle transition ;
- renderer initialization/ready/failure/destroyed ;
- surface failure ;
- device lost ;
- WGPU category observable selon section 13.7 ;
- audio initialization outcome et generic backend error.

Ne sont pas des événements : chaque frame, chaque present réussi, chaque resize intermédiaire, chaque playback ou chaque perte diagnostic. Ces faits utilisent des scalars remplaçables ou restent hors MVP.

Saturation : overwrite du plus ancien événement, avec `ever_wrapped`, intervalle d'ordres conservés et compteur saturant d'overwrites. Contention/ingress plein : drop du nouvel événement et compteur de drop best-effort. Aucun événement `DiagnosticsDropped` n'est ajouté au ring lui-même.

Un record ne contient aucun `Vec` ou `String` à croissance libre. Les textes opaques sont stockés dans une représentation physiquement bornée et accompagnés d'un flag `truncated`.

## 17. Concurrence et dégradation

Le contrat ne gèle pas la primitive (`try_lock`, atomics, bounded channel), mais gèle les propriétés :

- aucune acquisition bloquante sur CAPTURE ou degraded READ ;
- aucun retry non borné ;
- plusieurs sections indépendantes pour éviter qu'une contention GPU rende build/lifecycle illisibles ;
- aucun lock du store conservé pendant formatage ou écriture consumer ;
- aucun panic sur poison ; une section potentiellement incohérente devient `UNAVAILABLE` ;
- aucun custom lock-free MPMC sans justification expérimentale et preuve de publication de record complet ;
- async backend producers n'appellent pas le consumer et ne modifient pas directement l'état autoritatif moteur.

Si plusieurs failures surviennent, chaque section se dégrade indépendamment. La perte de toutes les sections est autorisée. Le runtime ne doit pas attendre qu'une observation soit lue ni qu'un consumer progresse.

## 18. Budgets numériques initiaux

Toutes les valeurs de cette section sont **PROPOSED / REQUIRES EXPERIMENT**. Elles sont des plafonds de design pour empêcher une implémentation ouverte, pas des performances déjà démontrées.

**FROZEN DECISION** : ces budgets sont acceptés comme gates provisoires obligatoires de la future implémentation. Ils ne constituent pas des performances promises. Si une gate n'est pas respectée, la réponse autorisée est `reduce scope/capacity` ou `request contract revision`; une mesure ne peut jamais justifier une augmentation silencieuse.

### 18.1 Mémoire et capacité

| Budget | Proposition initiale | Portée / justification |
|---|---:|---|
| store résident enabled | **64 KiB maximum par handle/runtime** | inclut boot facts, renderer records, ring, ingress, texte et metadata ; exclut les allocations internes préexistantes de WGPU/audio |
| observation matérialisée | **64 KiB maximum supplémentaire par read** | copie propriétaire bornée ; mémoire transitoire totale diagnostic cible ≤128 KiB |
| event ring | **64 records** | événements rares uniquement ; overwrite-oldest |
| taille physique event record | **320 bytes maximum** | ring ≤20 KiB ; inclut éventuel texte borné |
| renderer incarnations retenues | **8 au total** | couvre 2 actives actuelles + churn tool ; active records prioritaires |
| opaque text par champ | **192 bytes UTF-8 maximum** | truncation à une frontière UTF-8 valide + flag |
| opaque text agrégé résident | **16 KiB maximum** | toutes sections/ring confondus |
| async ingress | **8 records/slots par producer class** | burst court ; drop au-delà ; aucune croissance |
| disabled explicit handle | **2 KiB maximum** | mode + build facts, sans ring ni runtime records |

Si ces sous-budgets ne tiennent pas dans 64 KiB avec l'overhead réel, l'implémentation doit réduire les capacités ou demander une révision du contrat ; elle ne peut pas dépasser silencieusement le total.

### 18.2 Coût

| Opération | Proposition initiale | Contraintes absolues |
|---|---:|---|
| steady-state par frame | **≤0,5 µs p99 native ; ≤1 µs p99 WASM** | zéro allocation ; au plus mises à jour scalaires requises |
| capture rare main/runtime thread | **≤5 µs p99** | hors coût de l'opération moteur autoritative déjà exécutée ; aucune attente/I/O |
| capture callback backend | **≤2 µs p99** | aucune allocation proportionnelle au texte ; drop immédiat si indisponible |
| full normal materialization | **≤250 µs p99 native ; ≤500 µs p99 WASM** | maximum 64 events/8 renderers, aucune backend query |
| degraded/panic-thread read attempt | **≤100 µs p99** | try-only, partial return, aucun retry |

Ces seuils doivent être mesurés en debug et release pertinents ; l'architecture ne promet pas encore qu'ils sont atteignables sur chaque hôte. Un échec de budget entraîne réduction du MVP/capacités avant toute augmentation.

### 18.3 Allocation et compteurs

- Après bootstrap enabled, CAPTURE steady-state et événements rares MUST viser zéro allocation dynamique.
- READ/MATERIALIZE MAY allouer dans son plafond ; sous allocator failure, aucune observation finale n'est garantie.
- Tous les compteurs long-lived utilisent saturation + flag, jamais wrap ni panic debug.
- La longueur logique, la capacité allouée et les textes temporaires doivent tous être inclus dans la preuve de boundedness.

## 19. API publique et stabilité backend

L'API publique future expose uniquement des concepts GPE : build facts, lifecycle, role/incarnation, adapter/backend categories, surface categories, device-lost observation, audio summary, quality et stamps.

Elle MUST NOT exposer :

- `wgpu::Device`, `Adapter`, `Surface`, `Error` ou leurs lifetimes ;
- `rodio`, `cpal`, `web_sys` ou objets audio backend ;
- références mutables vers le store ;
- une enum fermée qui rend chaque nouvelle catégorie backend breaking sans `Unknown/Other` ou stratégie non exhaustive.

Le changement de backend interne reste possible tant que le nouvel adapter produit les mêmes concepts et marque les champs impossibles `NOT_APPLICABLE/UNKNOWN`. Une compatibilité sémantique est requise ; l'égalité des textes backend ne l'est pas.

## 20. Native et WASM

Le contrat s'applique aux deux cibles, avec availability explicite :

- tool renderer : `NOT_APPLICABLE` sur WASM ;
- renderer initialization : synchrone native actuellement, asynchrone WASM ; lifecycle partiel doit rester lisible ;
- adapter name/backend facts : peuvent être masqués/normalisés par le navigateur ; `UNKNOWN` est correct ;
- native audio et WebAudio ont des initialization outcomes différents mais la même enveloppe sémantique ;
- filesystem, signal, process exit et thread identity ne font pas partie du noyau commun ;
- le handle peut avoir une implémentation cfg différente, mais sa sémantique read-only/non-globale reste identique.

L'API ne doit pas imposer publiquement une primitive native si elle rend la cible WASM artificiellement stubbed. Inversement, la lecture native depuis un panic thread doit rester possible ; le choix `Arc`/`Rc` interne est un détail d'implémentation à valider.

## 21. Texte opaque et privacy

### Inclus par défaut dans le MVP

- adapter name borné et marqué opaque ;
- message device-lost facultatif et borné ;
- excerpt audio backend facultatif et borné.

### Exclu par défaut

- driver/driver info ;
- paths, audio assets, environment, hostname, username ;
- panic payload et game metadata, qui appartiennent au consumer.

Tronquer ne rend pas un texte safe-to-upload. Toute observation expose une distinction entre metadata contrôlée et texte opaque. GPE ne redige pas universellement et ne déclare jamais un rapport partageable. Consentement, redaction et upload appartiennent à PRESENT/consumer.

**FROZEN DECISION** : `adapter name` est collecté localement par défaut lorsqu'il est disponible. Il est stocké dans la limite opaque de la section 18, porte `opaque_text` et un indicateur `truncated` explicite, et n'est jamais qualifié `safe-to-upload`. Driver info reste hors MVP.

## 22. Failure behavior contract

| Failure diagnostic | Comportement requis |
|---|---|
| store/section contended | abandon ou section `UNAVAILABLE`; jamais attendre |
| ring/ingress plein | drop/overwrite selon section 16, loss counter best-effort |
| renderer record plein | évincer ended le plus ancien, sinon abandonner le nouveau record diagnostic |
| texte trop long | truncation avant croissance proportionnelle, `truncated=true` |
| compteur saturé | rester à max + saturated flag |
| materialization allocation failure | aucune observation garantie ; moteur non consulté/modifié |
| consumer formatter/writer failure | hors GPE ; aucune rétroaction |
| backend callback pendant teardown | attribuer à l'ancienne incarnation ou drop ; jamais à l'actuelle |
| lifecycle terminal non capturé | dernière phase reste last-observed/stale ; ne pas fabriquer `Ended` |
| diagnostics entièrement indisponibles | consumer reçoit une absence/degradation si possible ; perte totale acceptable |

## 23. Ce que le contrat ne garantit pas

- état atomique global du moteur ;
- état physique actuel du GPU, de la surface ou de l'audio ;
- livraison d'un callback backend ;
- thread ou lock context des callbacks non documentés ;
- causalité entre producers ;
- cause racine d'une erreur WGPU ;
- final report après panic, abort, kill, native crash ou OOM ;
- accès lorsque l'allocator, la mémoire ou le process est corrompu ;
- absence de mirror drift non détectable ;
- persistance, flush, filesystem ou upload ;
- performance avant validation expérimentale des budgets.

## 24. Décisions gelées

Le présent freeze gèle précisément :

1. abandon de `Snapshot` et adoption de l'observation partielle ;
2. handle explicite pré-`run`, par runtime, non global et read-only consumer ;
3. ownership consumer du panic hook et de toute présentation/persistence ;
4. séparation CAPTURE/READ/PRESENT ;
5. modèle de couverture L1/L2/L3 de la section 10, avec état unique partagé par L1/L2 et L3 hors core ;
6. maintien du default fatal WGPU par absence d'uncaptured handler dans le MVP ;
7. provenance renderer `(role, incarnation)` et non-réutilisation ;
8. modèle availability/freshness/representation kind ;
9. historique rare, borné, sans causalité revendiquée ;
10. périmètre exact des données MVP des sections 5 et 13 ;
11. résumé audio limité aux trois champs génériques ;
12. modes compile absent/runtime disabled/enabled/export consumer ;
13. nouvel entrypoint enabled explicite, sans builder général, avec `run(config, game)` historique conservé ;
14. feature Cargo `diagnostics`, default-off pour le MVP ;
15. adapter name local par défaut, opaque, borné et jamais safe-to-upload ; driver info hors MVP ;
16. privacy boundary et textes physiquement bornés ;
17. budgets de la section 18 acceptés comme gates provisoires obligatoires `PROPOSED / REQUIRES EXPERIMENT` ;
18. réduction de scope/capacité ou révision du contrat en cas de dépassement, jamais augmentation silencieuse ;
19. absence de préparation/implémentation de F01–F22 dans cette mission.

## 25. Décisions encore ouvertes

Il ne reste aucune décision humaine bloquante pour le freeze architectural MVP.

Les choix suivants restent ouverts sans rouvrir le contrat :

- nom Rust exact du nouvel entrypoint et des types handle/registration, sous réserve de conserver la forme gelée et de ne pas introduire un builder général ;
- résultats expérimentaux des budgets ; seuls une réduction ou une demande explicite de révision sont autorisées en cas d'échec ;
- éventuelle réévaluation future du default-off après campagnes A/B, par décision explicite ;
- stratégie future pour observer les uncaptured WGPU errors tout en changeant explicitement leur sémantique ; hors MVP ;
- format de présentation/JSON ; consumer-owned ;
- extension driver info, detailed metrics ou playback summary ; hors MVP ;
- intégration concrète de Void Canticle, absent de ce dépôt.

La stratégie F01–F22 et son harness sont différés à une mission ultérieure après implémentation du MVP ; ils n'ont été ni préparés ni exécutés ici.

## 26. Freeze record et conditions maintenues

Freeze enregistré en version **1.0**, le **2026-08-27**.

Les conditions de freeze sont satisfaites :

1. les quatre décisions humaines antérieurement ouvertes ont été résolues par écrit ;
2. L1/L2/L3 est explicite et conforme à la revue gelée ;
3. le statut, la date et la version de freeze sont inscrits ;
4. aucune contradiction avec la revue adversariale n'a été identifiée ;
5. le périmètre MVP et les exclusions sont acceptés sans collecte opportuniste ;
6. la décision WGPU Option A est confirmée pour le MVP ;
7. les budgets restent expérimentalement non validés malgré le freeze architectural ;
8. aucune implémentation n'est autorisée avant validation humaine explicite du contrat FROZEN résultant.

Après freeze, une modification sémantique de l'ownership, de la vérité, de l'identité, du périmètre ou des budgets nécessite une réouverture/version du contrat. Une correction factuelle démontrée peut être apportée avec justification sans réouvrir les décisions non affectées.

## 27. Final frozen contract

L'architecture minimale retenue est un store borné par runtime, accessible par un handle consumer explicite créé avant `run`, alimenté par GPE aux points autoritatifs et lu normalement en L1 ou en best-effort dégradé en L2. L1 et L2 partagent le même état ; L3 franchit la frontière du processus et reste hors GPE core. Le store expose des faits de build, lifecycle, renderer incarnations, adapter/backend, surface, last-present, device lost, catégories WGPU sans changement fatal et un résumé audio générique. Toute donnée porte disponibilité, fraîcheur et provenance ; l'historique annonce un ordre d'enregistrement partiel, jamais une causalité.

Le contrat architectural MVP est gelé. Ses budgets restent `PROPOSED / REQUIRES EXPERIMENT` et doivent être falsifiés pendant la future implémentation ; ce statut expérimental ne réouvre pas l'architecture.

**CONTRACT STATUS: FROZEN — VERSION 1.0 — 2026-08-27**
