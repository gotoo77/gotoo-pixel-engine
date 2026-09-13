import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { createFirstFrameTiming } from "../../web/diagnostics.js";

describe("unsupported WebGPU diagnostics UX", () => {
  it("terminates first-frame timing as UNSUPPORTED instead of aging into VERY SLOW", () => {
    let now = 100;
    const timing = createFirstFrameTiming({ now: () => now, slowThresholdMs: 3000 });

    now = 114;
    timing.observe("startup unsupported", "navigator.gpu.requestAdapter missing");
    now = 900_000;

    expect(timing.snapshot()).toMatchObject({
      status: "UNSUPPORTED",
      classification: "N/A",
      elapsedMs: 14,
      waitingFor: null,
      stalledForMs: 0,
      complete: false,
    });
  });

  it("builds a diagnostics URL from the current page while preserving query and fragment", async () => {
    const diagnosticsCore = await import("../../web/diagnostics-core.js");

    expect(diagnosticsCore.diagnosticsUrl).toBeTypeOf("function");
    expect(
      diagnosticsCore.diagnosticsUrl(
        "https://gotoo77.github.io/gotoo-pixel-engine/?foo=bar&diagnostics=0#arcade",
      ),
    ).toBe(
      "https://gotoo77.github.io/gotoo-pixel-engine/?foo=bar&diagnostics=1#arcade",
    );
  });

  it("offers an OPEN DIAGNOSTICS action in the WebGPU-unavailable notice", () => {
    const html = readFileSync(new URL("../../web/index.html", import.meta.url), "utf8");

    expect(html).toContain("OPEN DIAGNOSTICS");
    expect(html).toContain("diagnosticsUrl(globalThis.location?.href");
  });
});
