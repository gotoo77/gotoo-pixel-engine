import { safeString } from "./diagnostics-core.js";

export const DIAGNOSTICS_SCHEMA_VERSION = 2;

export function createStartupStateLatch(initialState = "page shell initializing") {
  let state = safeString(initialState, "page shell initializing");
  let terminal = null;
  let reason = null;

  function snapshot() {
    return { state, terminal, reason };
  }

  function set(nextState) {
    if (terminal !== null) return snapshot();
    const next = safeString(nextState);
    state = next;
    if (/^unsupported\b/i.test(next)) terminal = "unsupported";
    if (/^failed\b/i.test(next)) terminal = "failed";
    return snapshot();
  }

  function fail(failureReason, failedState = "failed — engine renderer initialization failed") {
    if (terminal !== null && terminal !== "failed") return snapshot();
    terminal = "failed";
    reason = safeString(failureReason, "startup failed");
    state = safeString(failedState, "failed");
    return snapshot();
  }

  function unsupported(unsupportedReason, unsupportedState = "unsupported — WebGPU unavailable") {
    if (terminal !== null) return snapshot();
    terminal = "unsupported";
    reason = safeString(unsupportedReason, "WebGPU unavailable");
    state = safeString(unsupportedState);
    return snapshot();
  }

  return { set, fail, unsupported, snapshot };
}

function normalizeFailureStage(category) {
  const stages = {
    CreateSurface: "create_surface",
    SurfaceConfiguration: "create_surface",
    RequestAdapter: "request_adapter",
    NoAdapter: "request_adapter",
    RequestDevice: "request_device",
    Device: "request_device",
    QueueSubmit: "first_submit",
    Present: "first_frame",
  };
  return category ? (stages[category] ?? "renderer") : null;
}

export function deriveEngineTriageFacts(snapshot) {
  const text = safeString(snapshot, "");
  const rendererMatch = text.match(
    /RendererRecord\s*\{[\s\S]*?lifecycle:\s*DiagnosticField\s*\{[\s\S]*?value:\s*Some\(\s*([A-Za-z0-9_]+)/,
  );
  const category = text.match(
    /last_wgpu_error:\s*DiagnosticField\s*\{[\s\S]*?value:\s*Some\(\s*([A-Za-z0-9_]+)/,
  )?.[1] ?? null;
  const rendererState = rendererMatch?.[1] ?? null;

  return {
    rendererState,
    failureCategory: category,
    failureStage: rendererState === "InitializationFailed" ? normalizeFailureStage(category) ?? "renderer" : null,
  };
}

export function deriveTriage({
  gpuFacts = {},
  engineFacts = {},
  firstFrame = {},
  wasmLoadMode = "unknown",
} = {}) {
  const adapterUsable = gpuFacts.adapterUsable;
  const firstFrameState = firstFrame.complete
    ? "reached"
    : ["FAILED", "UNSUPPORTED"].includes(firstFrame.status)
      ? "not reached"
      : "not observed";

  return {
    failureStage: engineFacts.failureStage ?? null,
    failureCategory: engineFacts.failureCategory ?? null,
    webGpuApi: gpuFacts.available ? "available" : "unavailable",
    browserAdapter:
      adapterUsable === true ? "available" : adapterUsable === false ? "unavailable" : "unknown",
    browserAdapterDetail: safeString(gpuFacts.adapter, "unknown"),
    browserAdapterSource: "browser probe",
    renderer: engineFacts.rendererState ?? "unknown",
    rendererSource: "engine",
    firstFrame: firstFrameState,
    wasmLoadMode: safeString(wasmLoadMode, "unknown"),
  };
}

function eventMs(events, label, { last = false } = {}) {
  const matches = events.filter((event) => event?.label === label);
  const event = last ? matches.at(-1) : matches[0];
  return Number.isFinite(Number(event?.ms)) ? Number(event.ms) : null;
}

function duration(events, startLabel, endLabel, options = {}) {
  const start = eventMs(events, startLabel, options);
  const end = eventMs(events, endLabel, options);
  if (start === null || end === null || end < start) return null;
  return end - start;
}

export function derivePhaseTimings(events = []) {
  const wasmInstantiateMs =
    duration(events, "WASM instantiateStreaming started", "WASM instantiateStreaming resolved") ??
    duration(events, "WASM instantiate started", "WASM instantiate resolved");

  return {
    moduleImportMs: duration(events, "Arcade module import started", "Arcade module import completed"),
    wasmInstantiateMs,
    requestAdapterMs: duration(events, "WebGPU requestAdapter started", "WebGPU requestAdapter resolved", {
      last: true,
    }),
    requestDeviceMs: duration(events, "WebGPU requestDevice started", "WebGPU requestDevice resolved", {
      last: true,
    }),
    failureAtMs: eventMs(events, "engine startup failed"),
    firstFrameAtMs:
      eventMs(events, "first post-submit requestAnimationFrame callback") ??
      eventMs(events, "first requestAnimationFrame callback"),
  };
}

export function createDiagnosticsRunId(random = Math.random) {
  const value = Math.floor(Math.max(0, Math.min(0.999999999, Number(random()) || 0)) * 0x1000000);
  return value.toString(16).toUpperCase().padStart(6, "0");
}

export function createJavaScriptErrorTracker() {
  let lastUnhandledError = null;
  let kind = null;

  function recordError(event) {
    kind = "error";
    lastUnhandledError = safeString(event?.error ?? event?.message ?? event, "unknown error");
    return snapshot();
  }

  function recordRejection(event) {
    kind = "unhandledrejection";
    lastUnhandledError = safeString(event?.reason ?? event, "unknown rejection");
    return snapshot();
  }

  function snapshot() {
    return { lastUnhandledError, kind };
  }

  return { recordError, recordRejection, snapshot };
}

function formatMaybeMs(value) {
  return value === null || value === undefined ? "not observed" : `${Math.round(value)} ms`;
}

export function formatDiagnosticsSummary({ runId, status = {}, triage = {}, timings = {}, javascript = {} } = {}) {
  return [
    `GPE WEB DIAGNOSTICS v${DIAGNOSTICS_SCHEMA_VERSION}`,
    `run: ${safeString(runId)}`,
    "",
    "Overall",
    `  ${safeString(status.marker, "[INFO]")} ${safeString(status.label, "INFO")}: ${safeString(status.reason, "diagnostics in progress")}`,
    "",
    "Triage",
    `  failure stage: ${safeString(triage.failureStage, "none")}`,
    `  failure category: ${safeString(triage.failureCategory, "none")}`,
    `  WebGPU API: ${safeString(triage.webGpuApi)} [browser]`,
    `  browser adapter: ${safeString(triage.browserAdapter)} [browser probe]`,
    `  browser adapter detail: ${safeString(triage.browserAdapterDetail)}`,
    `  renderer: ${safeString(triage.renderer)} [engine]`,
    `  first frame: ${safeString(triage.firstFrame)}`,
    `  WASM load mode: ${safeString(triage.wasmLoadMode)}`,
    "",
    "Timing",
    `  module import: ${formatMaybeMs(timings.moduleImportMs)}`,
    `  WASM instantiate: ${formatMaybeMs(timings.wasmInstantiateMs)}`,
    `  requestAdapter: ${formatMaybeMs(timings.requestAdapterMs)}`,
    `  requestDevice: ${formatMaybeMs(timings.requestDeviceMs)}`,
    `  failure detected: ${formatMaybeMs(timings.failureAtMs)}`,
    `  first frame: ${formatMaybeMs(timings.firstFrameAtMs)}`,
    "",
    "JavaScript",
    `  unhandled error: ${safeString(javascript.lastUnhandledError, "none")}`,
  ].join("\n");
}

export function buildDiagnosticsJson({
  runId,
  status = {},
  triage = {},
  timings = {},
  javascript = {},
  browser = null,
  canvas = null,
  startup = null,
  watchdog = null,
  firstFrame = null,
  timeline = null,
  engineObservation = null,
} = {}) {
  return {
    schema_version: DIAGNOSTICS_SCHEMA_VERSION,
    run_id: safeString(runId),
    status: {
      level: status.level ?? null,
      marker: status.marker ?? null,
      label: status.label ?? null,
      reason: status.reason ?? null,
    },
    triage: {
      failure_stage: triage.failureStage ?? null,
      failure_category: triage.failureCategory ?? null,
      webgpu_api: triage.webGpuApi ?? null,
      browser_adapter: triage.browserAdapter ?? null,
      browser_adapter_detail: triage.browserAdapterDetail ?? null,
      browser_adapter_source: triage.browserAdapterSource ?? null,
      renderer: triage.renderer ?? null,
      renderer_source: triage.rendererSource ?? null,
      first_frame: triage.firstFrame ?? null,
      wasm_load_mode: triage.wasmLoadMode ?? null,
    },
    timings,
    javascript,
    browser,
    canvas,
    startup,
    watchdog,
    first_frame_timing: firstFrame,
    timeline,
    engine_observation: engineObservation,
  };
}
