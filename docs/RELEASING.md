# Releasing Gotoo Pixel Engine

GPE uses Conventional Commits, Semantic Versioning and Cocogitto. Git history is
the source for the next version and `CHANGELOG.md`. crates.io publication is out
of scope.

## Local tools

```bash
cargo install cocogitto
cargo install cargo-edit
```

The repository targets Cocogitto 7.x. `cargo-edit` provides
`cargo set-version`. Web release validation also requires the same
`wasm32-unknown-unknown` and `wasm-bindgen` prerequisites as the existing Web CI.

## Commit policy

Accepted types:

```text
feat fix perf refactor docs test build ci chore style revert
```

Scopes are optional and intentionally unrestricted.

```text
feat(render): add image fit modes
fix(audio): restart looping music correctly
docs(api): document framebuffer lifecycle
feat(render)!: change framebuffer resize semantics
```

A `BREAKING CHANGE:` footer is equivalent to `!`.

```text
fix      -> PATCH
feat     -> MINOR
breaking -> MAJOR
```

Other non-breaking types do not directly bump versions. Cocogitto's native
`0.x` behavior never promotes `--auto` to `1.0.0`; the first stable `1.0.0`
must be an explicit human decision such as `cog bump --major`.

## Changelog policy

`CHANGELOG.md` uses Cocogitto's built-in `remote` template.

Visible: `feat`, `fix`, `perf`, `docs`, `revert`. Breaking commits keep their
semantic category and are explicitly marked by Cocogitto's native breaking
indicator.

Omitted as release noise: `refactor`, `test`, `build`, `ci`, `chore`, `style`.

No historical release entries are fabricated before the first GPE SemVer
baseline.

## Local commit hook

GPE already uses versioned `.githooks`. Preferred setup:

```bash
python scripts/dev.py install-hooks
```

The added `commit-msg` hook only executes:

```bash
cog verify --file "$1"
```

The same hook is declared in `cog.toml`, so Cocogitto can also install it:

```bash
cog install-hook commit-msg
```

Quick isolated checks:

```bash
cog verify "feat(render): add test feature"
cog verify "did some stuff"   # must fail
```

Hooks are bypassable; GitHub Actions remains authoritative.

## CI policy

The CI checks only non-merge commits introduced by the current PR or push to
`main`. This avoids retroactively validating GPE's pre-policy history while no
GPE SemVer baseline exists.

The currently observed merge model preserves branch commits behind a GitHub
merge commit, so every non-merge branch commit must be Conventional. Merge
commits themselves are ignored.

If GPE later becomes squash-only, the final squash commit/title must also be
Conventional. The current PR check remains stricter by validating intermediate
branch commits too; relax that only as an explicit policy change.

After the first SemVer baseline exists, local history validation is:

```bash
cog check --from-latest-tag
```

## Pre-bump gates

`cog bump` is restricted to `main`. Before Cocogitto creates a version commit or
tag it reuses the existing GPE validation surface:

```bash
python scripts/dev.py check
python scripts/dev.py check-web
python scripts/dev.py build-web
```

Only after those gates pass:

```bash
cargo set-version <version>
cargo check --locked
```

`Cargo.lock` is tracked and staged into the version commit. A manifest/lockfile
mismatch aborts the pre-bump phase. `post_bump_hooks` is empty: no push, GitHub
Release or registry publication is implicit.

## Preview and local release

Use only from clean, up-to-date `main` after the baseline tag exists:

```bash
git switch main
git pull --ff-only
cog check --from-latest-tag
cog bump --dry-run --auto
```

After reviewing the preview:

```bash
cog bump --auto
```

This creates the local version commit and `vX.Y.Z` tag. Remote publication stays
separate:

```bash
git push origin main
git push origin vX.Y.Z
```

Inspect and replace `vX.Y.Z` with the actual tag before pushing.

## Bootstrap: first GPE SemVer tag

At policy adoption, Cargo is `0.1.0`, no GPE `vX.Y.Z` tag exists, existing tags
are game/project milestones, and there are no GitHub Releases.

After this tooling change is merged to `main`, establish the human baseline:

```bash
git switch main
git pull --ff-only
git tag -a v0.1.0 -m "GPE v0.1.0 baseline"
git push origin v0.1.0
```

Do not use `cog bump` to manufacture this baseline.

## GitHub Release follow-up

A tag-triggered GitHub Release workflow is intentionally not activated before
the baseline. Otherwise the bootstrap `v0.1.0` tag would silently become a real
GitHub Release, coupling two operations this migration keeps separate.

The smallest follow-up after `v0.1.0` exists is one workflow that:

1. triggers only on `v*.*.*`;
2. grants only `contents: write`;
3. requires the tag commit to be reachable from `main`;
4. checks tag version == `Cargo.toml` package version;
5. reuses `scripts/dev.py check`, `check-web` and `build-web`;
6. derives notes with `cog changelog --at <tag>`;
7. creates the GitHub Release only after all gates pass;
8. never publishes to crates.io.

Cocogitto's default `[skip ci]` suffix is disabled in `cog.toml` so the future
tag workflow cannot be suppressed by its version commit.
