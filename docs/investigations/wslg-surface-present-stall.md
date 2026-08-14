# WSLg — crash de Weston selon la taille de surface

## Statut

Cause externe identifiée avec un niveau de confiance élevé : le processus
`weston` de WSLg segfault dans `libpixman` pendant la présentation de certaines
tailles de surface.

Le `Broken pipe` observé par Gotoo Pixel Engine est une conséquence de la mort
du compositeur Wayland, et non la cause initiale.

Environnement concerné : exécution native Linux sous WSLg.

Le problème a été découvert pendant le développement de Tetris, mais a été
reproduit après suppression de pratiquement toute la logique spécifique à
Tetris.

## Environnement reproduisant le problème

Versions relevées :

    WSL        2.4.11.0
    kernel     5.15.167.4-1
    WSLg       1.0.65
    MSRDC      1.2.5716
    Direct3D   1.611.1-81528511
    DXCore     10.0.26100.1-240331-1435.ge-release
    Windows    10.0.26100.8973

Session graphique :

    WAYLAND_DISPLAY=wayland-0
    DISPLAY=:0
    XDG_RUNTIME_DIR=/run/user/1000/

## Symptôme côté application

Certaines dimensions de fenêtre provoquent l'arrêt de l'application quelques
secondes après son lancement.

Erreur finale observée :

    Io error: Broken pipe (os error 32)
    Io error: Broken pipe (os error 32)
    Error: EngineError { message: "event loop failed: Exit Failure: 1" }

Des avertissements EGL/Mesa peuvent également apparaître :

    libEGL warning: failed to open /dev/dri/renderD128: Permission denied

ou :

    MESA-LOADER: failed to open vgem ...

Ces avertissements ne sont pas considérés comme la cause directe démontrée du
crash.

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

Cette liste décrit uniquement les dimensions effectivement testées. Elle ne
définit ni un seuil exact ni une règle générale sur les dimensions affectées.

## Élimination de la logique Tetris

Le problème reste reproductible lorsque `TetrisGame::update()` est réduit
temporairement à un simple effacement du framebuffer suivi de
`GameResult::Continue`.

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

## Preuve côté WSLg

Les logs WSLg montrent que le `Broken pipe` correspond à la disparition de
Weston :

    (EE) failed to read Wayland events: Broken pipe
    WSLGd: ... /usr/bin/weston ... terminated with signal 11

Le phénomène apparaît à répétition lors des reproductions.

Le noyau confirme explicitement le crash du compositeur et localise le fault
dans `libpixman` :

    weston[...]: segfault ... in libpixman-1.so.0.42.2
    potentially unexpected fatal signal 11
    Comm: weston

WSLg indique également que son chemin d'accélération glamor/GBM n'est pas
disponible et qu'il bascule vers le rendu logiciel :

    Missing Wayland requirements for glamor GBM backend
    Failed to initialize glamor, falling back to sw

Le crash observé dans `libpixman`, composant utilisé par le rendu/compositing
logiciel, est cohérent avec ce fallback. Cette cohérence ne permet cependant
pas, à elle seule, d'affirmer quel bug précis de WSLg/Weston/pixman est en
cause.

## Confirmation supplémentaire — Space Invaders, août 2026

Le même défaut a été recroisé pendant une investigation qui semblait initialement audio.

Space Invaders utilisait :

    window_width: 768
    window_height: 672

Sous WSLg, le jeu présentait rapidement :

- un warning audio `Buffer underrun/overrun` ;
- des sons perçus comme saccadés ou désynchronisés ;
- puis, sur certaines exécutions, le même `Broken pipe` et le même échec d'event loop que Tetris.

Plusieurs pistes audio ont été éliminées :

- un WAV continu joué directement par PulseAudio était propre ;
- un probe utilisant le vrai backend GPE/Rodio était propre ;
- les trois sons procéduraux exacts de Space Invaders étaient propres dans le probe ;
- le build `--release` ne changeait rien au symptôme.

Le seul changement de la taille de fenêtre vers la dimension déjà connue comme stable :

    960 × 612

fait disparaître le `Broken pipe` pendant le test et rend l'audio à nouveau comparable au comportement natif observé sur Mac.

Cette confirmation est importante : un symptôme apparemment audio peut être secondaire à un chemin de présentation WSLg défaillant. Voir également [`native-audio-xrun-spam.md`](native-audio-xrun-spam.md).

## Conclusion

La chaîne de panne observée est :

    Gotoo Pixel Engine / wgpu / winit
        -> présentation Wayland
        -> Weston / WSLg
        -> SIGSEGV dans libpixman
        -> connexion Wayland détruite
        -> Broken pipe côté application
        -> event loop failed

Le défaut observé n'est donc pas démontré comme un défaut de la logique Tetris,
du calcul de viewport ou du renderer de Gotoo Pixel Engine.

L'instrumentation montre au contraire que le moteur atteint correctement la
présentation avant la panne externe. Le ralentissement brutal de `present()`
dans le cas 960 × 960 est un symptôme utile de la défaillance du chemin de
présentation.

Cette investigation ne prétend pas identifier la ligne fautive dans Weston ou
pixman. Elle établit en revanche que, dans l'environnement testé, le processus
Weston de WSLg meurt par SIGSEGV et provoque ensuite l'erreur visible par le
moteur.

## Workaround temporaire

Tetris utilise provisoirement :

    window_width: 960,
    window_height: 612,

Cette dimension est connue comme stable dans l'environnement testé.

Space Invaders applique désormais le même workaround uniquement lorsqu'il détecte WSL via `WSL_DISTRO_NAME` :

    WSL    -> 960 × 612
    autres -> 768 × 672

Ce choix est explicitement un workaround WSLg et ne constitue pas un correctif
moteur. Il devra être supprimé dès que l'environnement WSLg concerné ne
reproduira plus le problème ou qu'une solution amont sera disponible.

## Tests complémentaires effectués

- désactivation/réduction de la logique Tetris : problème toujours présent ;
- framebuffer noir uniquement : problème toujours présent ;
- différentes tailles de framebuffer/fenêtre ;
- backend `WGPU_BACKEND=vulkan` : problème reproduit ;
- backend `WGPU_BACKEND=gl` : problème reproduit ;
- comparaison avec Snake ;
- comparaison Space Invaders WSLg / Mac natif ;
- test Space Invaders en debug et release ;
- probe audio séparé du rendu Space Invaders ;
- instrumentation détaillée du renderer ;
- inspection de `/mnt/wslg/stderr.log` ;
- inspection de `dmesg` ;
- relevé des versions WSL/WSLg/Windows.

## Suite

Le debugging du moteur sur ce symptôme peut s'arrêter ici tant qu'aucun élément
nouveau ne met en cause son code.

Actions utiles restantes :

- vérifier le comportement sur Linux natif et/ou sur une version WSLg plus
  récente ;
- rechercher un bug WSLg/Weston/pixman déjà connu correspondant à ce SIGSEGV ;
- préparer un reproducer minimal indépendant de Tetris si nécessaire ;
- ouvrir ou compléter une issue WSLg avec les dimensions stable/défaillantes,
  les versions, les logs Weston et la trace `dmesg` ;
- supprimer les workarounds 960 × 612 lorsque le problème amont est résolu.
