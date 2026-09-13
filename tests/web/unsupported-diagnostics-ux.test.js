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
});
