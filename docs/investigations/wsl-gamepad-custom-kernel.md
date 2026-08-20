# WSL — gamepad USB, noyau custom et `/dev/input`

## Statut

Résolu dans l’environnement testé.

Le contrôleur était correctement reconnu par Windows et pouvait être attaché à WSL via `usbipd`, mais le noyau WSL livré dans l’environnement ne contenait pas les sous-systèmes input/gamepad nécessaires. La solution validée a été de compiler un noyau Microsoft WSL custom avec `evdev`, `joydev`, `joystick` et `xpad`, puis de rendre disponibles les modules USB/IP correspondants.

Le résultat final est un contrôleur Xbox 360 visible dans WSL via :

```text
/dev/input/event0
/dev/input/js0
```

et détecté par `gilrs` / `gamepad_input_probe` dans GPE.

## Environnement observé

Avant modification :

```text
5.15.167.4-microsoft-standard-WSL2
```

Après compilation :

```text
6.18.40.1-microsoft-standard-WSL2-gpe+
```

Contrôleur côté Windows / USB :

```text
VID:PID 045e:028e
Contrôleur XBOX 360 pour Windows
```

Une fois attaché dans WSL :

```text
Microsoft X-Box 360 pad
Handlers=js0 event0
```

## Symptôme initial

GPE affichait :

```text
DEVICE NONE
WAITING FOR GAMEPAD
```

et WSL ne possédait même pas de répertoire :

```text
/dev/input
```

Pourtant Windows listait bien le périphérique avec `usbipd list`.

## Diagnostic du noyau WSL stock

Les contrôles suivants ont montré que le problème ne venait pas de `gilrs` mais du noyau :

```bash
uname -r
sudo modprobe xpad
find /lib/modules/$(uname -r) -iname 'xpad*.ko*' 2>/dev/null
zcat /proc/config.gz 2>/dev/null | grep -E \
'CONFIG_(INPUT|INPUT_EVDEV|INPUT_JOYDEV|INPUT_JOYSTICK|JOYSTICK_XPAD)'
```

Dans le noyau d’origine, les options utiles étaient absentes :

```text
CONFIG_INPUT=y
# CONFIG_INPUT_JOYDEV is not set
# CONFIG_INPUT_EVDEV is not set
# CONFIG_INPUT_JOYSTICK is not set
```

et :

```text
modprobe: FATAL: Module xpad not found
```

Le périphérique pouvait donc traverser USB/IP sans qu’un driver input Linux puisse créer `event*` ou `js*`.

## Construction du noyau WSL custom

Sources Microsoft : branche `linux-msft-wsl-6.18.y` du dépôt `microsoft/WSL2-Linux-Kernel`.

Les options activées dans la configuration WSL :

```bash
CONFIG=arch/x86/configs/config-wsl

./scripts/config --file "$CONFIG" \
  --set-str LOCALVERSION "-microsoft-standard-WSL2-gpe" \
  --enable INPUT_EVDEV \
  --enable INPUT_JOYDEV \
  --enable INPUT_JOYSTICK \
  --enable JOYSTICK_XPAD

make KCONFIG_CONFIG="$CONFIG" olddefconfig
```

Vérification :

```bash
grep -E \
'CONFIG_LOCALVERSION=|CONFIG_INPUT_EVDEV=|CONFIG_INPUT_JOYDEV=|CONFIG_INPUT_JOYSTICK=|CONFIG_JOYSTICK_XPAD=' \
"$CONFIG"
```

Résultat validé :

```text
CONFIG_LOCALVERSION="-microsoft-standard-WSL2-gpe"
CONFIG_INPUT_JOYDEV=y
CONFIG_INPUT_EVDEV=y
CONFIG_INPUT_JOYSTICK=y
CONFIG_JOYSTICK_XPAD=y
```

Le noyau a ensuite été compilé avec la configuration WSL. Une compilation complète a pris plusieurs dizaines de minutes sur la machine de test.

## Modules et artefacts WSL

Les modules ont d’abord été installés dans un staging directory :

```bash
make INSTALL_MOD_PATH="$PWD/modules" modules_install
```

Les headers et `perf` ont également été préparés :

```bash
make headers_install INSTALL_HDR_PATH="$PWD/headers"

make -C tools/perf \
  NO_JEVENTS=1 \
  NO_JVMTI=1 \
  NO_LIBTRACEEVENT=1 \
  install \
  DESTDIR="$PWD/perf" \
  prefix=/
```

La branche Microsoft utilisée fournit le script :

```text
Microsoft/scripts/gen_artifacts_vhdx.sh
```

Le script attend :

```text
<modules dir> <headers dir> <perf dir> <kernelversion> <output file>
```

Exemple :

```bash
sudo Microsoft/scripts/gen_artifacts_vhdx.sh \
  "$PWD/modules" \
  "$PWD/headers" \
  "$PWD/perf" \
  "$(make -s kernelrelease)" \
  "$PWD/modules.vhdx"
```

Les artefacts utiles étaient alors :

```text
arch/x86/boot/bzImage
modules.vhdx
```

Ils ont été copiés côté Windows, par exemple dans :

```text
C:\Users\<user>\wsl-kernels\
```

## `.wslconfig`

Configuration utilisée :

```ini
[wsl2]
kernel=C:\\Users\\<user>\\wsl-kernels\\bzImage-gpe
kernelModules=C:\\Users\\<user>\\wsl-kernels\\modules-gpe.vhdx
```

Puis dans PowerShell :

```powershell
wsl --shutdown
```

Après redémarrage de la distribution :

```bash
uname -r
```

doit afficher le noyau custom.

## Deuxième problème : `vhci_hcd` introuvable

Après démarrage sur le noyau custom, `usbipd attach` échouait encore :

```text
usbipd: error: Loading vhci_hcd failed; update with 'wsl --update'.
```

La configuration montrait pourtant :

```text
CONFIG_USBIP_CORE=m
CONFIG_USBIP_VHCI_HCD=m
```

mais :

```bash
sudo modprobe vhci_hcd
```

répondait :

```text
modprobe: FATAL: Module vhci_hcd not found in directory /lib/modules/<kernelrelease>
```

Le VHDX d’artefacts n’était donc pas suffisant dans l’environnement testé pour fournir les modules sous `/lib/modules/<kernelrelease>`.

Le workaround validé a été d’installer directement les modules dans la distribution WSL, depuis l’arbre déjà compilé :

```bash
cd ~/WSL2-Linux-Kernel
sudo make modules_install
```

Puis :

```bash
sudo modprobe vhci_hcd
```

sans erreur.

Vérification :

```bash
find /lib/modules/$(uname -r) \
  \( -name 'vhci-hcd.ko*' -o -name 'usbip-core.ko*' \) \
  -print
```

## Attacher le périphérique depuis Windows

Le `BUSID` peut changer après débranchement/rebranchement ; ne jamais le figer dans la documentation.

Dans PowerShell administrateur :

```powershell
usbipd list
```

Si nécessaire :

```powershell
usbipd bind --force --busid <BUSID>
```

Puis, avec une distribution WSL en cours d’exécution :

```powershell
usbipd attach --wsl --busid <BUSID>
```

Une fois l’attach réussi :

```powershell
usbipd list
```

doit indiquer `Attached`.

## Vérification côté WSL

```bash
lsusb
ls -l /dev/input
cat /proc/bus/input/devices
```

Résultat observé :

```text
Bus ... ID 045e:028e Microsoft Corp. Xbox360 Controller

/dev/input/event0
/dev/input/js0

N: Name="Microsoft X-Box 360 pad"
H: Handlers=js0 event0
```

## Troisième problème : permissions `/dev/input`

Les devices étaient créés avec :

```text
crw-rw---- root input ... event0
crw-rw---- root input ... js0
```

Le compte utilisateur n’était pas membre du groupe `input`, donc `gilrs` ne voyait toujours aucun gamepad.

Correction :

```bash
sudo usermod -aG input "$USER"
newgrp input
id
```

Après cela :

```bash
cargo run --example gamepad_input_probe
```

a détecté correctement le contrôleur et les événements de boutons.

## Chaîne fonctionnelle finale

```text
Contrôleur USB
    -> Windows
    -> usbipd-win
    -> USB/IP
    -> vhci_hcd dans WSL
    -> xpad
    -> Linux input
    -> /dev/input/event0 + /dev/input/js0
    -> gilrs
    -> GPE Input/Gamepad
```

## Ce qui n’était pas un bug GPE

- absence initiale de `/dev/input` ;
- absence de `xpad` dans le noyau WSL stock ;
- échec `vhci_hcd` quand les modules du noyau custom n’étaient pas installés ;
- permissions Unix du groupe `input`.

GPE ne pouvait recevoir aucun événement avant que toute cette chaîne soit opérationnelle.

## Note de maintenance

Un futur `wsl --update` peut remplacer le noyau/runtime WSL et modifier la situation. Avant de conserver ce noyau custom indéfiniment, retester périodiquement le noyau Microsoft standard : si `CONFIG_INPUT_EVDEV`, `CONFIG_INPUT_JOYSTICK` et `xpad` deviennent disponibles par défaut, le workaround custom pourra être supprimé.
