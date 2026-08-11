# WebGPU graceful failure — implementation prompt

We found a real-world Web compatibility failure in gotoo-pixel-engine.

## Context

On Firefox 153.0.3 / Fedora 44 / Wayland with an AMD Radeon R7 200 Series
(Bonaire, Mesa radeonsi 26.1.6), the deployed Snake demo displays only a
black page.

In the browser console:

```text
navigator.gpu
-> undefined
```

Firefox `about:support` reports WEBGPU as `available`, but no WebGPU adapter
is exposed to page JavaScript.

The current `web/snake.html` bootstrap imports `./pkg/snake_web.js` and simply
calls:

```js
init();
```

The resulting rejected initialization promise currently leaves the user
with a black page and an opaque wasm-bindgen message:

```text
Using exceptions for control flow, don't mind me.
This isn't actually an error!
```

## Goal

Make Web demo startup fail gracefully when WebGPU is unavailable or WASM
initialization fails.

This is capability/error handling only.

Do NOT:

- implement a WebGL fallback;
- change renderer architecture;
- change native rendering;
- change game logic;
- modify audio work;
- broaden the current milestone unnecessarily.

## Requirements

1. Inspect the current web bootstrap for:
   - `web/snake.html`
   - `web/index.html`
   - any other Web entry point currently tracked in the repository.

2. Before starting the WASM application, detect whether WebGPU is exposed:

   ```js
   navigator.gpu
   ```

3. If WebGPU is unavailable:
   - do not invoke the WASM application;
   - replace the blank/black page with a small readable diagnostic;
   - explain that gotoo-pixel-engine currently requires WebGPU;
   - state that WebGPU is unavailable in this browser/device configuration;
   - do not claim that the browser itself never supports WebGPU.

4. Also handle rejection/errors from `init()`.
   A WASM/wgpu initialization failure must produce a useful visible message
   rather than an unhandled promise rejection and black page.

5. Keep detailed technical information in `console.error()` when useful, but
   keep the user-facing message concise.

6. Avoid duplicating bootstrap/error-display logic between demos if a tiny
   shared helper is appropriate. However, do not introduce an unnecessary
   framework or abstraction layer just for this.

7. Preserve the current visual behavior when WebGPU is available.

8. Add tests where practical for any extracted pure capability/error-formatting
   logic. Do not introduce browser automation solely for this change unless
   the repository already has such infrastructure.

9. Update documentation only if the current README claims Web support without
   mentioning the WebGPU requirement. Keep the documentation change concise.

## Validation

Run the repository's normal validation suite, including at minimum:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Also validate the `wasm32` target/build using the repository's existing Web
build procedure.

Report:

- files changed;
- exact behavior when `navigator.gpu` is undefined;
- exact behavior when `init()` rejects;
- validation commands/results.

Do not commit or push yet.

Before changing anything, inspect the current implementation and briefly
state the minimal change you intend to make.
