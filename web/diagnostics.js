import {
  createBoundedTimeline,
  diagnosticsRequested,
  formatTimeline,
  safeString,
  snapshotUnavailableDuringStartup,
} from "./diagnostics-core.js";
import { classifyStartupElapsed, createStartupWatchdog } from "./startup-watchdog.js";

const REFRESH_MS = 1000;
const MAX_TIMELINE_EVENTS = 64;
const DEFAULT_FIRST_FRAME_SLOW_MS = 3000;
const tracedAdapters = new WeakSet();
const tracedQueues = new WeakSet();

export { diagnosticsRequested };

function canvasFacts() {
  const canvas = document.querySelector("canvas");
  if (!canvas) {
    return {
      backing: "not created",
      css: "not created",
    };
  }

  const rect = canvas.getBoundingClientRect();
  return {
    backing: `${canvas.width} x ${canvas.height}`,
    css: `${Math.round(rect.width)} x ${Math.round(rect.height)}`,
  };
}

export function createFirstFrameTiming({
  now = () => globalThis.performance?.now?.() ?? Date.now(),
  slowThresholdMs = DEFAULT_FIRST_FRAME_SLOW_MS,
} = {}) {
  if (!Number.isFinite(slowThresholdMs) || slowThresholdMs <= 0) {
    throw new RangeError("slowThresholdMs must be a positive finite number");
  }

  const startedAt = now();
  let deviceReadyMs = null;
  let firstRafCallbackMs = null;
  let firstSubmitMs = null;
  let postSubmitRafMs = null;

  function elapsed() {
    return Math.max(0, now() - startedAt);
  }

  function observe(label) {
    const observedMs = elapsed();
    switch (String(label)) {
      case "WebGPU requestDevice resolved":
        if (deviceReadyMs === null) deviceReadyMs = observedMs;
        break;
      case "first requestAnimationFrame callback":
        if (firstRafCallbackMs === null) firstRafCallbackMs = observedMs;
        break;
      case "first GPUQueue.submit":
        if (firstSubmitMs === null) firstSubmitMs = observedMs;
        break;
      case "first post-submit requestAnimationFrame callback":
        if (postSubmitRafMs === null) postSubmitRafMs = observedMs;
        break;
      default:
        break;
    }
  }

  function snapshot() {
    const sampledElapsedMs = postSubmitRafMs ?? elapsed();
    const complete = postSubmitRafMs !== null;
    let waitingFor = null;
    let stalledForMs = 0;

    if (!complete && deviceReadyMs !== null && firstSubmitMs === null) {
      waitingFor = "first GPUQueue.submit";
      stalledForMs = Math.max(0, sampledElapsedMs - deviceReadyMs);
    } else if (!complete && firstSubmitMs !== null) {
      waitingFor = "first post-submit requestAnimationFrame callback";
      stalledForMs = Math.max(0, sampledElapsedMs - firstSubmitMs);
    }

    let status = complete ? "complete" : "monitoring";
    if (!complete && waitingFor !== null) {
      status = stalledForMs >= slowThresholdMs ? "SLOW FIRST FRAME" : `waiting for ${waitingFor}`;
    }

    return {
      status,
      classification: classifyStartupElapsed(sampledElapsedMs),
      elapsedMs: sampledElapsedMs,
      slowThresholdMs,
      waitingFor,
      stalledForMs,
      deviceReadyMs,
      firstRafCallbackMs,
      firstSubmitMs,
      postSubmitRafMs,
      deviceToSubmitMs:
        deviceReadyMs !== null && firstSubmitMs !== null
          ? Math.max(0, firstSubmitMs - deviceReadyMs)
          : null,
      submitToPostRafMs:
        firstSubmitMs !== null && postSubmitRafMs !== null
          ? Math.max(0, postSubmitRafMs - firstSubmitMs)
          : null,
      complete,
    };
  }

  return { observe, snapshot };
}

export function instrumentGpuQueueSubmit(
  queue,
  markEvent,
  {
    traced = tracedQueues,
    requestAnimationFrameFn =
      typeof globalThis.requestAnimationFrame === "function"
        ? globalThis.requestAnimationFrame.bind(globalThis)
        : null,
  } = {},
) {
  if (!queue || typeof queue.submit !== "function" || traced.has(queue)) return;
  traced.add(queue);

  const original = queue.submit.bind(queue);
  let firstSubmit = true;
  try {
    Object.defineProperty(queue, "submit", {
      configurable: true,
      value: (...args) => {
        if (!firstSubmit) return original(...args);
        firstSubmit = false;
        markEvent("first GPUQueue.submit");
        const result = original(...args);
        if (typeof requestAnimationFrameFn === "function") {
          markEvent("first post-submit requestAnimationFrame scheduled");
          requestAnimationFrameFn(() => {
            markEvent("first post-submit requestAnimationFrame callback");
          });
        } else {
          markEvent(
            "first post-submit requestAnimationFrame unavailable",
            "requestAnimationFrame missing",
          );
        }
        return result;
      },
    });
  } catch (error) {
    markEvent("GPUQueue.submit trace unavailable", safeString(error));
  }
}

export function instrumentAdapterRequestDevice(
  adapter,
  markEvent,
  traced = tracedAdapters,
  queueTraceOptions = {},
) {
  if (!adapter || typeof adapter.requestDevice !== "function" || traced.has(adapter)) return;
  traced.add(adapter);

  const original = adapter.requestDevice.bind(adapter);
  try {
    Object.defineProperty(adapter, "requestDevice", {
      configurable: true,
      value: async (...args) => {
        markEvent("WebGPU requestDevice started");
        try {
          const device = await original(...args);
          markEvent("WebGPU requestDevice resolved");
          instrumentGpuQueueSubmit(device?.queue, markEvent, queueTraceOptions);
          return device;
        } catch (error) {
          markEvent("WebGPU requestDevice rejected", safeString(error));
          throw error;
        }
      },
    });
  } catch (error) {
    markEvent("WebGPU requestDevice trace unavailable", safeString(error));
  }
}

export function installWebGpuApiTrace(
  gpu,
  markEvent,
  traced = tracedAdapters,
  queueTraceOptions = {},
) {
  if (!gpu || typeof gpu.requestAdapter !== "function") {
    markEvent("WebGPU API trace unavailable", "navigator.gpu.requestAdapter missing");
    return;
  }

  const original = gpu.requestAdapter.bind(gpu);
  try {
    Object.defineProperty(gpu, "requestAdapter", {
      configurable: true,
      value: async (...args) => {
        markEvent("WebGPU requestAdapter started");
        try {
          const adapter = await original(...args);
          markEvent(
            "WebGPU requestAdapter resolved",
            adapter ? "adapter selected" : "null adapter",
          );
          instrumentAdapterRequestDevice(adapter, markEvent, traced, queueTraceOptions);
          return adapter;
        } catch (error) {
          markEvent("WebGPU requestAdapter rejected", safeString(error));
          throw error;
        }
      },
    });
    markEvent("WebGPU API trace installed", "requestAdapter/requestDevice/first-submit");
  } catch (error) {
    markEvent("WebGPU API trace unavailable", safeString(error));
  }
}

async function browserGpuFacts(requestAdapter) {
  if (typeof requestAdapter !== "function") {
    return {
      available: false,
      adapter: "unavailable",
    };
  }

  try {
    const adapter = await requestAdapter();
    if (!adapter) {
      return {
        available: true,
        adapter: "requestAdapter returned null",
      };
    }

    let info = null;
    try {
      info = adapter.info ?? null;
    } catch (error) {
      return {
        available: true,
        adapter: `selected; info read failed: ${safeString(error)}`,
      };
    }

    if (!info) {
      return {
        available: true,
        adapter: "selected; info unavailable",
      };
    }

    const parts = [];
    for (const key of ["vendor", "architecture", "device", "description"]) {
      const value = info[key];
      if (value) parts.push(`${key}=${value}`);
    }
    return {
      available: true,
      adapter: parts.length ? parts.join(", ") : "selected; info empty",
    };
  } catch (error) {
    return {
      available: true,
      adapter: `probe failed: ${safeString(error)}`,
    };
  }
}

export function deriveDiagnosticStatus({ startupError = null, watchdog = {}, firstFrame = {} } = {}) {
  if (startupError || watchdog.outcome === "failed" || watchdog.status === "FAILED") {
    return { level: "error", label: "ERROR", marker: "[ERROR]", reason: "startup failed" };
  }

  if (watchdog.outcome === "unsupported" || watchdog.status === "UNSUPPORTED") {
    return {
      level: "warn",
      label: "ATTENTION",
      marker: "[WARN]",
      reason: "WebGPU unsupported in this environment",
    };
  }

  const slowStartup =
    watchdog.status === "SLOW STARTUP DETECTED" ||
    watchdog.status === "COMPLETE AFTER SLOW STARTUP" ||
    ["SUSPICIOUS", "SLOW", "VERY SLOW"].includes(watchdog.classification);
  if (slowStartup) {
    return {
      level: "warn",
      label: "ATTENTION",
      marker: "[WARN]",
      reason: "startup timing needs attention",
    };
  }

  const slowFirstFrame =
    firstFrame.status === "SLOW FIRST FRAME" ||
    (firstFrame.complete && ["SUSPICIOUS", "SLOW", "VERY SLOW"].includes(firstFrame.classification));
  if (slowFirstFrame) {
    return {
      level: "warn",
      label: "ATTENTION",
      marker: "[WARN]",
      reason: "first frame timing needs attention",
    };
  }

  if (
    watchdog.outcome === "success" &&
    watchdog.status === "complete" &&
    watchdog.classification === "FAST" &&
    firstFrame.complete &&
    firstFrame.classification === "FAST"
  ) {
    return {
      level: "ok",
      label: "OK",
      marker: "[OK]",
      reason: "startup and first frame complete",
    };
  }

  return {
    level: "info",
    label: "INFO",
    marker: "[INFO]",
    reason: "diagnostics in progress",
  };
}

export function applyDiagnosticStatus(panel, status) {
  if (panel?.root?.dataset) panel.root.dataset.severity = safeString(status?.level, "info");
  if (panel?.statusBadge) {
    panel.statusBadge.textContent = `${safeString(status?.marker, "[INFO]")} ${safeString(status?.label, "INFO")}`;
  }
}

function pad2(value) {
  return String(value).padStart(2, "0");
}

export function diagnosticsFilename(date = new Date()) {
  const year = date.getUTCFullYear();
  const month = pad2(date.getUTCMonth() + 1);
  const day = pad2(date.getUTCDate());
  const hour = pad2(date.getUTCHours());
  const minute = pad2(date.getUTCMinutes());
  const second = pad2(date.getUTCSeconds());
  return `gpe-web-diagnostics-${year}${month}${day}-${hour}${minute}${second}.txt`;
}

export function saveDiagnosticsText(
  text,
  {
    documentRef = globalThis.document,
    urlApi = globalThis.URL,
    BlobCtor = globalThis.Blob,
    date = new Date(),
  } = {},
) {
  if (!documentRef || typeof documentRef.createElement !== "function") {
    throw new Error("document unavailable for diagnostics export");
  }
  if (!urlApi || typeof urlApi.createObjectURL !== "function") {
    throw new Error("URL.createObjectURL unavailable for diagnostics export");
  }
  if (typeof BlobCtor !== "function") {
    throw new Error("Blob unavailable for diagnostics export");
  }

  const filename = diagnosticsFilename(date);
  const blob = new BlobCtor([String(text)], { type: "text/plain;charset=utf-8" });
  const url = urlApi.createObjectURL(blob);
  const anchor = documentRef.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  if (anchor.style) anchor.style.display = "none";
  documentRef.body?.appendChild?.(anchor);
  try {
    anchor.click();
  } finally {
    anchor.remove?.();
    urlApi.revokeObjectURL?.(url);
  }
  return filename;
}

function makePanel() {
  const root = document.createElement("aside");
  root.id = "gpe-web-diagnostics";
  root.dataset.severity = "info";
  root.setAttribute("aria-label", "GPE Web diagnostics");
  root.innerHTML = `
    <style>
      #gpe-web-diagnostics {
        position: fixed;
        z-index: 10000;
        top: 10px;
        left: 10px;
        width: min(680px, calc(100vw - 20px));
        max-height: calc(100vh - 20px);
        box-sizing: border-box;
        overflow: auto;
        padding: 10px;
        border: 1px solid #8b949e;
        border-radius: 4px;
        background: #071018f2;
        color: #d8f3ff;
        box-shadow: 0 4px 24px #000a;
        font: 12px/1.35 ui-monospace, SFMono-Regular, Consolas, monospace;
        white-space: pre-wrap;
      }
      #gpe-web-diagnostics[data-severity="ok"] { border-color: #3fb950; }
      #gpe-web-diagnostics[data-severity="warn"] { border-color: #d29922; }
      #gpe-web-diagnostics[data-severity="error"] { border-color: #f85149; }
      #gpe-web-diagnostics[data-severity="info"] { border-color: #8b949e; }
      #gpe-web-diagnostics header {
        display: flex;
        gap: 8px;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
      }
      #gpe-web-diagnostics .gpe-diagnostics-title,
      #gpe-web-diagnostics .gpe-diagnostics-actions {
        display: flex;
        gap: 8px;
        align-items: center;
      }
      #gpe-web-diagnostics strong { color: #8bd5ff; }
      #gpe-web-diagnostics [data-role="status-badge"] {
        padding: 2px 5px;
        border: 1px solid currentColor;
        border-radius: 3px;
        color: #8b949e;
        font-weight: 700;
      }
      #gpe-web-diagnostics[data-severity="ok"] [data-role="status-badge"] { color: #3fb950; }
      #gpe-web-diagnostics[data-severity="warn"] [data-role="status-badge"] { color: #d29922; }
      #gpe-web-diagnostics[data-severity="error"] [data-role="status-badge"] { color: #f85149; }
      #gpe-web-diagnostics[data-severity="info"] [data-role="status-badge"] { color: #8b949e; }
      #gpe-web-diagnostics button {
        border: 1px solid #8bd5ff;
        border-radius: 3px;
        background: #102330;
        color: #d8f3ff;
        padding: 3px 7px;
        font: inherit;
        cursor: pointer;
      }
      #gpe-web-diagnostics pre { margin: 0; font: inherit; }
    </style>
    <header>
      <div class="gpe-diagnostics-title">
        <strong>GPE WEB DIAGNOSTICS</strong>
        <span data-role="status-badge">[INFO] INFO</span>
      </div>
      <div class="gpe-diagnostics-actions">
        <button type="button" data-action="copy">COPY</button>
        <button type="button" data-action="save">SAVE TXT</button>
      </div>
    </header>
    <pre>initializing diagnostics…</pre>
  `;
  document.body.appendChild(root);
  return {
    root,
    output: root.querySelector("pre"),
    copy: root.querySelector('[data-action="copy"]'),
    save: root.querySelector('[data-action="save"]'),
    statusBadge: root.querySelector('[data-role="status-badge"]'),
  };
}

function formatTiming(value) {
  return value === null ? "not observed" : `${Math.round(value)} ms`;
}

export function installGpeWebDiagnostics() {
  if (!diagnosticsRequested(globalThis.location?.search ?? "")) return null;

  const panel = makePanel();
  const timeline = createBoundedTimeline({ maxEvents: MAX_TIMELINE_EVENTS });
  const firstFrameTiming = createFirstFrameTiming();
  let snapshotProvider = null;
  let startupState = "page shell initializing";
  let startupError = null;
  let gpuFacts = {
    available: Boolean(navigator.gpu),
    adapter: "probing…",
  };
  let startupWatchdog = null;

  function buildReport() {
    const canvas = canvasFacts();
    const watchdog = startupWatchdog.snapshot();
    const firstFrame = firstFrameTiming.snapshot();
    const status = deriveDiagnosticStatus({ startupError, watchdog, firstFrame });
    let engine = "unavailable (consumer did not expose a diagnostics snapshot)";
    if (snapshotProvider) {
      try {
        engine = safeString(snapshotProvider(), "no snapshot yet");
      } catch (error) {
        engine = snapshotUnavailableDuringStartup(startupState)
          ? "not available yet (WASM initialization in progress)"
          : `snapshot read failed: ${safeString(error)}`;
      }
    }

    const text = [
      "GPE WEB DIAGNOSTICS",
      "",
      "Overall status",
      `  ${status.marker} ${status.label}: ${status.reason}`,
      "",
      "Browser",
      `  userAgent: ${safeString(navigator.userAgent)}`,
      `  platform: ${safeString(navigator.userAgentData?.platform ?? navigator.platform)}`,
      `  language: ${safeString(navigator.language)}`,
      `  WebGPU available: ${gpuFacts.available}`,
      `  browser GPU adapter probe: ${gpuFacts.adapter}`,
      "",
      "Canvas / display",
      `  backing size: ${canvas.backing}`,
      `  CSS size: ${canvas.css}`,
      `  devicePixelRatio: ${safeString(globalThis.devicePixelRatio)}`,
      `  viewport: ${globalThis.innerWidth ?? "unknown"} x ${globalThis.innerHeight ?? "unknown"}`,
      "",
      "Startup",
      `  state: ${startupState}`,
      `  error: ${startupError ?? "none observed by page shell"}`,
      "",
      "Startup watchdog",
      `  status: ${watchdog.status}`,
      `  classification: ${watchdog.classification}`,
      `  threshold: ${watchdog.thresholdMs} ms without progress`,
      `  elapsed: ${Math.round(watchdog.elapsedMs)} ms`,
      `  last milestone: ${watchdog.lastMilestone}`,
      `  no progress for: ${Math.round(watchdog.stalledForMs)} ms`,
      `  last slow stall: ${watchdog.lastSlowDurationMs === null ? "none" : `${Math.round(watchdog.lastSlowDurationMs)} ms`}`,
      "",
      "First frame timing",
      `  status: ${firstFrame.status}`,
      `  classification: ${firstFrame.classification}`,
      `  slow threshold: ${firstFrame.slowThresholdMs} ms after the last first-frame milestone`,
      `  elapsed: ${Math.round(firstFrame.elapsedMs)} ms`,
      `  waiting for: ${firstFrame.waitingFor ?? "none"}`,
      `  stalled for: ${Math.round(firstFrame.stalledForMs)} ms`,
      `  device ready: ${formatTiming(firstFrame.deviceReadyMs)}`,
      `  first requestAnimationFrame callback: ${formatTiming(firstFrame.firstRafCallbackMs)}`,
      `  first GPUQueue.submit: ${formatTiming(firstFrame.firstSubmitMs)}`,
      `  device -> first submit: ${formatTiming(firstFrame.deviceToSubmitMs)}`,
      `  first post-submit requestAnimationFrame callback: ${formatTiming(firstFrame.postSubmitRafMs)}`,
      `  first submit -> post-submit RAF: ${formatTiming(firstFrame.submitToPostRafMs)}`,
      "",
      "Startup timeline",
      formatTimeline(timeline.snapshot()),
      "",
      "GPE engine observation",
      engine,
      "",
      "Notes",
      "  Browser adapter data is a separate JS probe; it is not claimed to be the adapter selected by GPE/wgpu.",
      "  WebGPU API tracing is diagnostics-only and may slightly perturb timing; use it to locate long waits, not to benchmark absolute latency.",
      "  GPUQueue.submit is a browser-side proxy for first rendering work; GPE engine diagnostics remain authoritative for renderer lifecycle and presents.",
      "  Startup/first-frame classifications: FAST <1s; SUSPICIOUS 1-3s; SLOW 3-8s; VERY SLOW >=8s.",
      "  The startup watchdog emits SLOW STARTUP DETECTED after 3s without a diagnostics milestone and records the recovery milestone.",
      "  Status meaning is also written as [OK], [WARN], [ERROR], or [INFO]; color is supplementary only.",
      "  Unknown data is intentionally left unknown rather than inferred.",
    ].join("\n");

    return { text, status };
  }

  function render() {
    const current = buildReport();
    panel.output.textContent = current.text;
    applyDiagnosticStatus(panel, current.status);
  }

  function rawMarkEvent(label, detail = null) {
    timeline.mark(label, detail);
    firstFrameTiming.observe(label, detail);
    render();
  }

  function markEvent(label, detail = null) {
    startupWatchdog?.observe(label, detail);
    rawMarkEvent(label, detail);
  }

  startupWatchdog = createStartupWatchdog({ emit: rawMarkEvent });

  const browserProbeRequestAdapter =
    typeof navigator.gpu?.requestAdapter === "function"
      ? navigator.gpu.requestAdapter.bind(navigator.gpu)
      : null;

  markEvent("diagnostics installed");
  installWebGpuApiTrace(navigator.gpu, markEvent);
  markEvent("browser GPU adapter probe started", `navigator.gpu=${Boolean(navigator.gpu)}`);

  browserGpuFacts(browserProbeRequestAdapter).then((facts) => {
    gpuFacts = facts;
    markEvent("browser GPU adapter probe finished", facts.adapter);
  });

  panel.copy.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(buildReport().text);
      panel.copy.textContent = "COPIED";
    } catch {
      panel.copy.textContent = "COPY FAILED";
    }
    setTimeout(() => {
      panel.copy.textContent = "COPY";
    }, 1500);
  });

  panel.save.addEventListener("click", () => {
    try {
      saveDiagnosticsText(buildReport().text);
      panel.save.textContent = "SAVED";
    } catch {
      panel.save.textContent = "SAVE FAILED";
    }
    setTimeout(() => {
      panel.save.textContent = "SAVE TXT";
    }, 1500);
  });

  const timer = setInterval(render, REFRESH_MS);
  globalThis.addEventListener(
    "pagehide",
    () => {
      clearInterval(timer);
      startupWatchdog.dispose();
    },
    { once: true },
  );
  render();

  return {
    markEvent,
    setSnapshotProvider(provider) {
      snapshotProvider = typeof provider === "function" ? provider : null;
      markEvent(
        snapshotProvider ? "engine snapshot provider attached" : "engine snapshot provider cleared",
      );
    },
    setStartupState(state) {
      startupState = safeString(state);
      markEvent("startup state changed", startupState);
    },
    setStartupError(error) {
      startupError = safeString(error);
      startupState = "failed";
      markEvent("startup error", startupError);
    },
  };
}
