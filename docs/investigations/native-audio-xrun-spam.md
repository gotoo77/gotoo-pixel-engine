# Audio natif — spam `Buffer underrun/overrun` et faux diagnostic Space Invaders

## Statut

Résolu pour le spam de logs ; diagnostic clarifié pour Space Invaders sous WSLg.

Deux phénomènes différents avaient été mélangés :

1. Rodio/CPAL signalait de façon répétitive des xruns récupérables (`Buffer underrun/overrun`) ;
2. Space Invaders présentait en plus un son apparemment décalé/saccadé sous WSL, mais la cause réelle était un problème de présentation WSLg dépendant de la taille de fenêtre.

## Symptôme initial

Lors du lancement de plusieurs exemples natifs sous WSL :

```text
audio stream error: Buffer underrun/overrun occurred.
audio stream error: Buffer underrun/overrun occurred.
audio stream error: Buffer underrun/overrun occurred.
...
```

Le flux pouvait continuer à fonctionner malgré ces messages.

## Source du spam

Le message ne venait pas d’un `eprintln!` GPE répété dans la boucle de jeu. Il provenait du callback d’erreur de la pile Rodio/CPAL.

Dans la version utilisée, `StreamError::BufferUnderrun` peut être remonté alors que le stream audio reste vivant. Le callback par défaut rendait chaque occurrence visible, ce qui transformait un événement récupérable en spam terminal.

## Correctif GPE retenu

Le backend natif installe désormais son propre callback :

- le premier xrun affiche un warning explicite ;
- les xruns suivants sont supprimés pour le reste du process ;
- toutes les autres erreurs stream restent affichées normalement.

Comportement attendu :

```text
audio stream warning: buffer underrun/overrun occurred; suppressing repeats
```

puis silence pour les répétitions.

Le code est dans `src/audio.rs`, avec un `AtomicBool` global au backend natif.

Le correctif a été mergé via la PR #11.

## Pourquoi ne pas ignorer toutes les erreurs audio

Les xruns sont souvent récupérables. À l’inverse, une erreur de device, de backend ou de stream invalidé peut nécessiter un vrai diagnostic.

Le filtrage est donc volontairement ciblé sur :

```text
StreamError::BufferUnderrun
```

et ne transforme pas le backend en trou noir à logs.

## Fausse piste : réduire arbitrairement le buffer

Une branche d’investigation a testé un buffer CPAL/Rodio fixe de 2048 frames :

```rust
.with_buffer_size(BufferSize::Fixed(2048))
```

Résultat observé sur Space Invaders : aucun changement perceptible.

Cette modification n’a pas été retenue. Réduire le buffer peut améliorer la latence, mais augmente aussi le risque d’underrun ; sans preuve qu’il corrigeait le symptôme, il n’y avait aucune raison de l’imposer au moteur.

## Test 1 — vérifier WSLg/PulseAudio indépendamment de GPE

Le serveur audio WSLg observé :

```text
Server String: unix:/mnt/wslg/PulseServer
Server Name: pulseaudio
Default Sink: RDPSink
```

Un WAV mono 44,1 kHz continu de cinq secondes a été généré puis joué avec :

```bash
paplay /tmp/gpe-audio-test.wav
```

Résultat : son parfaitement continu.

Conclusion : la chaîne générale WSLg/PulseAudio/RDP était capable de reproduire un flux PCM propre.

## Test 2 — probe GPE/Rodio minimal

Un `audio_probe` temporaire a ensuite utilisé le vrai chemin GPE :

```text
SoundBank -> NativeAudio -> Rodio mixer -> CPAL -> WSLg
```

Le probe jouait :

- un sinus continu de cinq secondes ;
- un blip court synchronisé avec un flash visuel sur `Space`.

Résultat : son propre et flash/blip synchronisés.

Conclusion : le backend GPE/Rodio n’introduisait pas, à lui seul, la désynchronisation observée dans Space Invaders.

## Test 3 — mêmes samples que Space Invaders

Le probe a ensuite synthétisé exactement les sons procéduraux utilisés par Space Invaders :

- explosion alien ;
- impact bunker ;
- explosion joueur.

Ils ont été testés séparément et en rafale.

Résultat : tous propres et synchronisés dans le probe.

Conclusion : les samples eux-mêmes n’étaient pas en cause.

## Comparaison de plateforme

Space Invaders utilisait les mêmes sons avec le même moteur :

- Mac mini / Linux natif : audio correct ;
- WSLg : audio perçu comme saccadé ou décalé, warning xrun rapide, puis parfois `Broken pipe`.

Le passage en build `--release` n’a rien changé.

Cela a éliminé l’hypothèse d’une simple surcharge du build debug.

## Cause réelle du « bug audio Space Invaders »

L’investigation WSLg existante a alors été recroisée.

Space Invaders ouvrait une fenêtre :

```text
768 x 672
```

Or l’environnement WSLg testé possède un défaut dépendant de la taille de surface : `present()` peut fortement bloquer, Weston finit par segfault dans `libpixman`, puis l’application reçoit :

```text
Io error: Broken pipe (os error 32)
Error: EngineError { message: "event loop failed: Exit Failure: 1" }
```

Space Invaders a été retesté avec une taille déjà connue comme stable :

```text
960 x 612
```

Résultat :

- plus de `Broken pipe` pendant le test ;
- comportement audio redevenu normal, comparable au Mac ;
- gameplay inchangé.

Le faux « bug audio » était donc au moins principalement un symptôme d’un chemin de présentation WSLg instable, et non un défaut de synchronisation de `SoundBank` ou de Rodio.

Voir : [`wslg-surface-present-stall.md`](wslg-surface-present-stall.md).

## Workaround actuellement conservé

Space Invaders choisit sa taille de fenêtre en fonction de l’environnement :

```text
WSL -> 960 x 612
ailleurs -> 768 x 672
```

La détection se fait via `WSL_DISTRO_NAME`.

Le workaround est volontairement limité à WSL afin de ne pas modifier la présentation sur les plateformes natives où `768 x 672` fonctionne correctement.

## Leçon de diagnostic

Le message :

```text
audio stream warning: buffer underrun/overrun occurred
```

ne prouve pas à lui seul que la cause d’un défaut audiovisuel est le backend audio.

Pour isoler correctement la panne :

1. tester la sortie audio système indépendamment de GPE ;
2. tester un flux continu dans GPE ;
3. tester les mêmes one-shots que le jeu dans un probe ;
4. comparer debug/release et plusieurs plateformes ;
5. regarder aussi la stabilité de la boucle graphique et du compositor.

Dans ce cas précis, ces étapes ont évité d’ajouter un réglage de buffer global injustifié au moteur.
