# WSLg — blocage de présentation selon la taille de surface

## Statut

Investigation en cours.

Environnement concerné : exécution native Linux sous WSLg.

Le problème a été découvert pendant le développement de Tetris, mais a été
reproduit après suppression de pratiquement toute la logique spécifique à
Tetris.

## Symptôme

Certaines dimensions de fenêtre provoquent l'arrêt de l'application quelques
secondes après son lancement.

Erreur finale observée :

    Io error: Broken pipe (os error 32)
    Io error: Broken pipe (os error 32)
    Error: EngineError { message: "event loop failed: Exit Failure: 1" }

Un avertissement EGL peut également apparaître :

    libEGL warning: failed to open /dev/dri/renderD128: Permission denied

ou, selon l'environnement :

    MESA-LOADER: failed to open vgem ...

Cet avertissement n'est pas considéré à ce stade comme la cause démontrée du
problème.

## Reproduction

Framebuffer Tetris :

    220 × 224

Surfaces testées comme provoquant le problème :

- 220 × 204
- 320 × 224
- 224 × 224
- 220 × 220
- 660 × 500
- 612 × 960
- 960 × 960

Surfaces observées comme stables :

- 960 × 300
- 960 × 612

Cette liste décrit uniquement les dimensions effectivement testées.
Elle ne définit pas un seuil exact.

## Élimination de la logique Tetris

Le problème reste reproductible lorsque `TetrisGame::update()` est réduit
temporairement à :

    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        frame.framebuffer.clear(...);
        GameResult::Continue
    }

Ont ainsi été éliminés comme causes nécessaires :

- input Tetris ;
- gravité ;
- déplacement et rotation ;
- collisions ;
- rendu spécifique Tetris ;
- logique métier Tetris.

Snake ne reproduit pas le problème dans sa configuration habituelle.

## Instrumentation du renderer

Le chemin de rendu a été instrumenté autour de :

1. `queue.write_texture`
2. `surface.get_current_texture`
3. encodage
4. `queue.submit`
5. `queue.present`

### Cas défaillant : surface 960 × 960

Extrait significatif :

    FRAME 7 AFTER WRITE    ~0.34 ms
    FRAME 7 AFTER ACQUIRE  ~0.47 ms
    FRAME 7 AFTER ENCODE   ~0.60 ms
    FRAME 7 AFTER SUBMIT   ~2.06 ms
    FRAME 7 AFTER PRESENT  ~1.53 s

Immédiatement après :

    Io error: Broken pipe (os error 32)

### Cas stable : surface 960 × 300

Après initialisation, `present()` reste généralement autour de 10–13 ms
pendant plusieurs dizaines de frames.

Aucun `Broken pipe` observé pendant le test.

## Conclusion actuelle

Le problème devient observable à la frontière de présentation GPU.

Il n'est pas démontré que `wgpu` soit lui-même responsable.

Les composants encore susceptibles d'être impliqués incluent notamment :

- wgpu ;
- backend graphique utilisé par wgpu ;
- Mesa ;
- WSLg ;
- compositeur / présentation de surface ;
- interaction winit avec l'environnement WSLg.

Le diagnostic ne permet pas encore de choisir entre ces causes.

## Workaround temporaire

Tetris utilise provisoirement :

    window_width: 960,
    window_height: 612,

Cette dimension est connue comme stable dans l'environnement testé.

Ce workaround ne constitue pas un correctif moteur.

## Tests complémentaires déjà effectués

- désactivation/réduction de la logique Tetris : problème toujours présent ;
- framebuffer noir uniquement : problème toujours présent ;
- différentes tailles de framebuffer/fenêtre ;
- backend `WGPU_BACKEND=vulkan` : problème reproduit ;
- backend `WGPU_BACKEND=gl` : problème reproduit ;
- comparaison avec Snake ;
- instrumentation détaillée du renderer.

## Suite de l'investigation

- relever précisément les versions `wgpu`, `winit`, Mesa et WSLg ;
- rechercher les issues connues correspondantes ;
- déterminer le backend réellement sélectionné ;
- construire si nécessaire un reproducer moteur minimal indépendant de Tetris ;
- vérifier le comportement hors WSLg ;
- supprimer le workaround 960 × 612 lorsque la cause sera comprise.