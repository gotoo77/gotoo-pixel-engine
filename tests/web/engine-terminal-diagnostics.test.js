import { describe, expect, it } from "vitest";

import {
  createFirstFrameTiming,
  detectEngineStartupFailure,
} from "../../web/diagnostics.js";
import { createStartupWatchdog } from "../../web/startup-watchdog.js";

const FAILED_RENDERER_SNAPSHOT = `DiagnosticObservation {
  runtime: DiagnosticSection {
    value: Some(
      RuntimeState {
        lifecycle: DiagnosticField {
          value: Some(
            ShuttingDown,
          ),
        },
      },
    ),
  },
  renderers: DiagnosticSection {
    value: Some(
      RendererObservations {
        records: [
          Some(
            RendererRecord {
              lifecycle: DiagnosticField {
                value: Some(
                  InitializationFailed,
                ),
              },
              last_wgpu_error: DiagnosticField {
                value: Some(
                  CreateSurface,
                ),
              },
            },
          ),
        ],
      },
    ),
  },
}`;

describe("engine terminal startup failure detection", () => {
  it("detects Renderer InitializationFailed and keeps the wgpu failure category", () => {
    expect(detectEngineStartupFailure(FAILED_RENDERER_SNAPSHOT)).toEqual({
      failed: true,
      reason: "renderer initialization failed (CreateSurface)",
    });
  });

  it("does not classify a healthy renderer snapshot as failed", () => {
    const healthy = FAILED_RENDERER_SNAPSHOT
      .replace("InitializationFailed", "Ready")
      .replace("CreateSurface", "Unknown");

    expect(detectEngineStartupFailure(healthy)).toEqual({
      failed: false,
      reason: null,
    });
  });
});

describe("terminal failure timing", () => {
  it("makes the startup watchdog terminal FAILED instead of continuing as slow", () => {
    let now = 0;
    const scheduled = [];
    const watchdog = createStartupWatchdog({
      now: () => now,
      schedule(callback) {
        scheduled.push(callback);
        return scheduled.length;
      },
      cancel() {},
    });

    watchdog.observe("startup state changed", "event loop running (winit handoff)");
    now = 4000;
    scheduled.at(-1)?.();
    now = 4200;
    watchdog.observe("engine startup failed", "renderer initialization failed (CreateSurface)");
    now = 20000;

    expect(watchdog.snapshot()).toMatchObject({
      status: "FAILED",
      classification: "N/A",
      outcome: "failed",
      complete: true,
      lastMilestone: "engine startup failed",
      elapsedMs: 4200,
      stalledForMs: 0,
    });
  });

  it("makes first-frame timing terminal FAILED and freezes elapsed time", () => {
    let now = 0;
    const timing = createFirstFrameTiming({ now: () => now });

    now = 900;
    timing.observe("first requestAnimationFrame scheduled");
    now = 1200;
    timing.observe("engine startup failed", "renderer initialization failed (CreateSurface)");
    now = 20000;

    expect(timing.snapshot()).toMatchObject({
      status: "FAILED",
      classification: "N/A",
      elapsedMs: 1200,
      waitingFor: null,
      stalledForMs: 0,
      complete: false,
    });
  });
});
