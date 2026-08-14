# Investigations techniques GPE

Ce dossier conserve les problèmes de plateforme et d’intégration rencontrés pendant le développement de Gotoo Pixel Engine. Le but n’est pas de produire une documentation utilisateur exhaustive, mais de garder une trace reproductible des symptômes, des fausses pistes, des causes confirmées et des workarounds.

## Investigations

- [`wslg-surface-present-stall.md`](wslg-surface-present-stall.md) — crash Weston/WSLg dépendant de la taille de surface, `present()` très lent puis `Broken pipe`.
- [`wsl-gamepad-custom-kernel.md`](wsl-gamepad-custom-kernel.md) — contrôleur USB visible par Windows mais absent de `/dev/input` sous WSL ; recompilation d’un noyau WSL avec evdev/joydev/xpad et configuration USB/IP.
- [`native-audio-xrun-spam.md`](native-audio-xrun-spam.md) — spam `Buffer underrun/overrun` de Rodio/CPAL sous WSL, diagnostic audio et séparation d’un faux problème audio Space Invaders d’un problème WSLg de présentation.
- [`linux-gamepad-centered-dpad.md`](linux-gamepad-centered-dpad.md) — D-pad exposé comme axe centré asymétrique par certains contrôleurs Linux, polarité verticale et suppression des faux événements numériques.

## Règle de diagnostic

Quand un symptôme apparaît dans un jeu, ne pas conclure immédiatement qu’il vient du jeu ou du moteur. Les investigations de ce dossier ont montré plusieurs cas où un symptôme visible dans GPE provenait d’une couche plus basse : WSLg/Weston, noyau WSL, USB/IP, ALSA/CPAL ou représentation matérielle d’un contrôleur.

La méthode retenue est systématiquement : réduire le cas, construire un probe minimal, comparer plusieurs plateformes et ne conserver dans GPE qu’un correctif justifié par un consommateur réel.
