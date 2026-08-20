# Native game packaging

GPE packages native games as ordinary archives that can be copied to a machine
without Rust, Cargo or the repository.

The packaging boundary deliberately stays in `scripts/dev.py`: it describes the
Cargo example, public binary name and runtime files required by each packaged
game. It is not an asset manager or a second build system.

## Void Canticle vertical slice

Build the package on the target operating system:

```bash
python scripts/dev.py package-native void-canticle
```

Supported targets for this first slice are native x86_64 Windows and Linux.
The command always performs a release build and produces one of:

```text
dist/native/void-canticle-windows-x86_64.zip
dist/native/void-canticle-linux-x86_64.tar.gz
```

The archive root contains:

```text
Windows
├── VoidCanticle.exe
├── LICENSE
└── assets/void_canticle/ui/choice/...

Linux
├── void_canticle
├── LICENSE
└── assets/void_canticle/ui/choice/...
```

The VC2.8 choice assets remain optional at runtime because the game has
procedural/synthesized fallbacks, but they are packaged so release behavior does
not silently diverge when real PNG/WAV overrides are added.

## Linux runtime contract

The Linux archive is self-contained with respect to GPE, Rust, Cargo and the
repository, but it deliberately does not bundle the host graphics/audio/input
stack. It therefore expects the normal runtime libraries of a Linux desktop.

The X11 path used by the portability smoke test requires `libxkbcommon-x11.so.0`,
provided by the Debian/Ubuntu package:

```bash
sudo apt install libxkbcommon-x11-0
```

This dependency is loaded dynamically by the window/input stack, so `ldd` alone
cannot discover it. The extracted-package launch smoke is intentionally kept in
addition to the `ldd` check for this reason.

`xvfb` is installed only by CI to provide a virtual X server; it is a test
harness dependency, not a Void Canticle runtime dependency for an ordinary
desktop session.

## GitHub Actions

`.github/workflows/native-packages.yml` builds on the native GitHub-hosted
runners:

- `windows-latest` for Windows x86_64;
- `ubuntu-latest` for Linux x86_64.

Each build archive is uploaded as a workflow artifact. A separate smoke job runs
on a fresh runner, does not checkout the repository and does not install Rust.
It downloads the archive, installs only the declared Linux runtime/test
prerequisites when applicable, extracts the archive, verifies the runtime assets
and launches the packaged executable. The Linux smoke also checks `ldd` for
unresolved directly linked runtime libraries.

The smoke window intentionally terminates a still-running game after a few
seconds. Exiting before that window is treated as a packaging/runtime failure.

## Releases

A tag matching:

```text
void-canticle-v*
```

runs the same package and smoke jobs. When both platforms pass, the workflow
creates (or updates on a rerun) the matching GitHub Release and attaches:

```text
void-canticle-windows-x86_64.zip
void-canticle-linux-x86_64.tar.gz
```

The tag is the game release version; the GPE crate version in `Cargo.toml` is not
used as the Void Canticle version.

## Adding another GPE game

Only add a `NATIVE_PACKAGES` entry after the game has a real standalone Cargo
example. Describe its public archive slug, platform binary names and runtime
paths, then extend the workflow matrix/release trigger when that game actually
needs distribution.

Do not introduce a generic asset pipeline, plugin system or `xtask` merely for
packaging.
