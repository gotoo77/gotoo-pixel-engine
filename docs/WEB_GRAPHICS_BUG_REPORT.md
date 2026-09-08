# GPE Web graphics bug report

Use this checklist when GPE Web shows a black screen, partial rendering, corrupted presentation, or another browser/GPU-specific failure.

## Shareable diagnostics mode

Open the failing GPE Arcade URL with:

```text
?diagnostics=1
```

If the URL already has query parameters, append:

```text
&diagnostics=1
```

The page displays a **GPE WEB DIAGNOSTICS** panel. Use **COPY** and attach the copied text to the bug report.

The page-level report includes, when available:

- browser user agent, platform and language;
- whether the browser exposes WebGPU;
- a separately labelled browser-side adapter probe;
- canvas backing size and CSS size;
- `devicePixelRatio` and viewport size;
- startup errors observed by the page shell;
- the consumer-provided GPE engine diagnostics observation.

The browser-side adapter probe is not presented as the adapter selected by GPE/wgpu. Unknown values remain unknown rather than being inferred.

## Linux information

Also provide:

```bash
lspci | grep -Ei 'vga|3d|display'
echo $XDG_SESSION_TYPE
```

For Firefox, open `about:support` and include the relevant information from:

- **Graphics**;
- **WebGPU**;
- **Compositing**;
- **Window Protocol**.

Record the exact browser version and retest with the current stable browser when practical.

## Minimum reproduction matrix

When practical, report whether the same build behaves the same way in:

- Firefox and Chromium on the affected operating system;
- the same browser on another operating system;
- Wayland and X11, when both sessions are available.

Do not change GPE renderer policy merely to make diagnostics succeed. Diagnostics must remain best-effort and must not fabricate unavailable engine facts.
