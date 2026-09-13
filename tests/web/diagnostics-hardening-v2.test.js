import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import * as diagnostics from "../../web/diagnostics.js";

function requireExport(name) {
  expect(typeof diagnostics[name], `${name} should be exported`).toBe("function");
  return diagnostics[name];
}

const FAILED_RENDERER_SNAPSHOT = `DiagnosticObservation {
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

describe("diagnostics v2 terminal state", () => {
  it("latches an engine failure so a later winit state cannot overwrite it", () => {
    const createStartupStateLatch = requireExport("createStartupStateLatch");
    const state = createStartupStateLatch("initializing Arcade WASM");

    state.fail("renderer initialization failed (CreateSurface)");
    state.set("event loop running (winit handoff)");

    expect(state.snapshot()).toEqual({
      state: "failed — engine renderer initialization failed",
      terminal: "failed",
      reason: "renderer initialization failed (CreateSurface)",
    });
  });

  it("latches unsupported startup too", () => {
    const createStartupStateLatch = requireExport("createStartupStateLatch");
    const state = createStartupStateLatch("page shell initializing");

    state.set("unsupported — WebGPU unavailable");
    state.set("importing Arcade module");

    expect(state.snapshot()).toMatchObject({
      state: "unsupported — WebGPU unavailable",
      terminal: "unsupported",
    });
  });
});

describe("diagnostics v2 triage", () => {
  it("normalizes the Firefox Android CreateSurface failure with provenance", () => {
    const deriveEngineTriageFacts = requireExport("deriveEngineTriageFacts");
    const deriveTriage = requireExport("deriveTriage");

    const engine = deriveEngineTriageFacts(FAILED_RENDERER_SNAPSHOT);
    expect(engine).toEqual({
      rendererState: "InitializationFailed",
      failureCategory: "CreateSurface",
      failureStage: "create_surface",
    });

    expect(
      deriveTriage({
        gpuFacts: {
          available: true,
          adapterUsable: false,
          adapter: "requestAdapter returned null",
        },
        engineFacts: engine,
        firstFrame: { complete: false, status: "FAILED" },
        wasmLoadMode: "streaming",
      }),
    ).toEqual({
      failureStage: "create_surface",
      failureCategory: "CreateSurface",
      webGpuApi: "available",
      browserAdapter: "unavailable",
      browserAdapterDetail: "requestAdapter returned null",
      browserAdapterSource: "browser probe",
      renderer: "InitializationFailed",
      rendererSource: "engine",
      firstFrame: "not reached",
      wasmLoadMode: "streaming",
    });
  });

  it("finds a failed renderer even when an earlier retained renderer is healthy", () => {
    const deriveEngineTriageFacts = requireExport("deriveEngineTriageFacts");
    const snapshot = `RendererObservations {
      records: [
        Some(RendererRecord {
          lifecycle: DiagnosticField { value: Some(Ready), },
          last_wgpu_error: DiagnosticField { value: Some(Unknown), },
        }),
        Some(RendererRecord {
          lifecycle: DiagnosticField { value: Some(InitializationFailed), },
          last_wgpu_error: DiagnosticField { value: Some(CreateSurface), },
        }),
      ],
    }`;

    expect(deriveEngineTriageFacts(snapshot)).toEqual({
      rendererState: "InitializationFailed",
      failureCategory: "CreateSurface",
      failureStage: "create_surface",
    });
  });

  it("distinguishes API exposure from usable adapter availability", () => {
    const deriveTriage = requireExport("deriveTriage");
    const triage = deriveTriage({
      gpuFacts: { available: true, adapterUsable: false, adapter: "requestAdapter returned null" },
      engineFacts: { rendererState: null, failureCategory: null, failureStage: null },
      firstFrame: { complete: false, status: "monitoring" },
      wasmLoadMode: "streaming",
    });

    expect(triage.webGpuApi).toBe("available");
    expect(triage.browserAdapter).toBe("unavailable");
  });

  it.each([
    ["WASM response.arrayBuffer rejected", "wasm"],
    ["WASM instantiateStreaming rejected", "wasm"],
    ["WebGPU requestAdapter rejected", "request_adapter"],
    ["WebGPU requestDevice rejected", "request_device"],
    ["GPUQueue.submit trace unavailable", "first_submit"],
    ["first post-submit requestAnimationFrame unavailable", "first_frame"],
  ])("normalizes %s as %s when no engine category is available", (label, stage) => {
    const deriveTriage = requireExport("deriveTriage");
    const triage = deriveTriage({
      gpuFacts: { available: true, adapterUsable: true, adapter: "selected" },
      engineFacts: { rendererState: null, failureCategory: null, failureStage: null },
      firstFrame: { complete: false, status: "FAILED" },
      wasmLoadMode: "streaming",
      events: [{ ms: 120, label, detail: "boom" }],
    });
    expect(triage.failureStage).toBe(stage);
  });
});

describe("diagnostics v2 timing summary", () => {
  it("derives phase durations from the startup timeline", () => {
    const derivePhaseTimings = requireExport("derivePhaseTimings");
    const timings = derivePhaseTimings([
      { ms: 46, label: "Arcade module import started" },
      { ms: 329, label: "Arcade module import completed" },
      { ms: 721, label: "WASM instantiateStreaming started" },
      { ms: 976, label: "WASM instantiateStreaming resolved" },
      { ms: 1046, label: "engine startup failed" },
    ]);

    expect(timings).toMatchObject({
      moduleImportMs: 283,
      wasmInstantiateMs: 255,
      failureAtMs: 1046,
    });
  });

  it("summarizes fetch/read, winit, adapter, device, submit and first-frame timings when observed", () => {
    const derivePhaseTimings = requireExport("derivePhaseTimings");
    const timings = derivePhaseTimings([
      { ms: 100, label: "WASM Chunked fetch started" },
      { ms: 140, label: "WASM Chunked fetch resolved" },
      { ms: 145, label: "WASM response.body reader started" },
      { ms: 300, label: "WASM response.body completed" },
      { ms: 310, label: "WASM instantiate started" },
      { ms: 360, label: "WASM instantiate resolved" },
      { ms: 370, label: "winit event loop control-flow handoff" },
      { ms: 400, label: "WebGPU requestAdapter started" },
      { ms: 420, label: "WebGPU requestAdapter resolved" },
      { ms: 425, label: "WebGPU requestDevice started" },
      { ms: 455, label: "WebGPU requestDevice resolved" },
      { ms: 490, label: "first GPUQueue.submit" },
      { ms: 510, label: "first post-submit requestAnimationFrame callback" },
    ]);

    expect(timings).toMatchObject({
      wasmFetchMs: 40,
      wasmReadMs: 155,
      wasmInstantiateMs: 50,
      winitHandoffAtMs: 370,
      requestAdapterMs: 20,
      requestDeviceMs: 30,
      deviceToFirstSubmitMs: 35,
      firstSubmitToFirstFrameMs: 20,
      firstFrameAtMs: 510,
    });
  });
});

describe("diagnostics v2 identity and export", () => {
  it("creates a stable six-digit hexadecimal run id from an injected random source", () => {
    const createDiagnosticsRunId = requireExport("createDiagnosticsRunId");
    expect(createDiagnosticsRunId(() => 0)).toBe("000000");
    expect(createDiagnosticsRunId(() => 0.5)).toMatch(/^[0-9A-F]{6}$/);
  });

  it("supports versioned JSON filenames", () => {
    expect(diagnostics.diagnosticsFilename(new Date("2026-09-13T06:30:45.000Z"), "json")).toBe(
      "gpe-web-diagnostics-20260913-063045.json",
    );
  });

  it("builds schema v2 JSON and a compact human summary", () => {
    const buildDiagnosticsJson = requireExport("buildDiagnosticsJson");
    const formatDiagnosticsSummary = requireExport("formatDiagnosticsSummary");
    const payload = {
      runId: "A7F3C2",
      status: { marker: "[ERROR]", label: "ERROR", reason: "startup failed" },
      triage: {
        failureStage: "create_surface",
        failureCategory: "CreateSurface",
        webGpuApi: "available",
        browserAdapter: "unavailable",
        browserAdapterDetail: "requestAdapter returned null",
        browserAdapterSource: "browser probe",
        renderer: "InitializationFailed",
        rendererSource: "engine",
        firstFrame: "not reached",
        wasmLoadMode: "streaming",
      },
      timings: { moduleImportMs: 283, wasmInstantiateMs: 255, failureAtMs: 1046 },
      javascript: { lastUnhandledError: null },
    };

    expect(buildDiagnosticsJson(payload)).toMatchObject({
      schema_version: 2,
      run_id: "A7F3C2",
      triage: { failure_stage: "create_surface", wasm_load_mode: "streaming" },
    });

    const summary = formatDiagnosticsSummary(payload);
    expect(summary).toContain("GPE WEB DIAGNOSTICS v2");
    expect(summary).toContain("run: A7F3C2");
    expect(summary).toContain("failure stage: create_surface");
    expect(summary).toContain("WebGPU API: available [browser]");
    expect(summary).toContain("renderer: InitializationFailed [engine]");
    expect(summary).not.toContain("DiagnosticObservation {");
  });
});

describe("diagnostics v2 JavaScript errors", () => {
  it("keeps the last unhandled error or rejection without consuming the browser event", () => {
    const createJavaScriptErrorTracker = requireExport("createJavaScriptErrorTracker");
    const tracker = createJavaScriptErrorTracker();

    tracker.recordError({ message: "surface exploded", error: new Error("surface exploded") });
    expect(tracker.snapshot().lastUnhandledError).toContain("surface exploded");

    tracker.recordRejection({ reason: new Error("promise exploded") });
    expect(tracker.snapshot().lastUnhandledError).toContain("promise exploded");
  });
});

describe("diagnostics v2 panel UX", () => {
  it("offers summary/full copy, TXT/JSON export and mobile wrapping", () => {
    const source = readFileSync(new URL("../../web/diagnostics.js", import.meta.url), "utf8");

    expect(source).toContain("COPY SUMMARY");
    expect(source).toContain("COPY FULL");
    expect(source).toContain("SAVE JSON");
    expect(source).toContain("flex-wrap");
    expect(source).toContain("overflow-wrap");
    expect(source).toContain('addEventListener("error"');
    expect(source).toContain('addEventListener("unhandledrejection"');
  });
});
