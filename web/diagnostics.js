import {
  createBoundedTimeline,
  diagnosticsRequested,
  formatTimeline,
  safeString,
  snapshotUnavailableDuringStartup,
} from "./diagnostics-core.js";

const REFRESH_MS = 1000;
const MAX_TIMELINE_EVENTS = 64;
const tracedAdapters = new WeakSet();

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

function instrumentAdapterRequestDevice(adapter, markEvent) {
  if (!adapter || typeof adapter.requestDevice !== "function" || tracedAdapters.has(adapter)) return;
  tracedAdapters.add(adapter);

  const original = adapter.requestDevice.bind(adapter);
  try {
    Object.defineProperty(adapter, "requestDevice", {
      configurable: true,
      value: async (...args) => {
        markEvent("WebGPU requestDevice started");
        try {
          const device = await original(...args);
          markEvent("WebGPU requestDevice resolved");
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

function installWebGpuApiTrace(markEvent) {
  const gpu = navigator.gpu;
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
          instrumentAdapterRequestDevice(adapter, markEvent);
          return adapter;
        } catch (error) {
          markEvent("WebGPU requestAdapter rejected", safeString(error));
          throw error;
        }
      },
    });
    markEvent("WebGPU API trace installed", "requestAdapter/requestDevice");
  } catch (error) {
    markEvent("WebGPU API trace unavailable", safeString(error));
  }
}

async function browserGpuFacts() {
  const gpu = navigator.gpu;
  if (!gpu) {
    return {
      available: false,
      adapter: "unavailable",
    };
  }

  try {
    const adapter = await gpu.requestAdapter();
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

function makePanel() {
  const root = document.createElement("aside");
  root.id = "gpe-web-diagnostics";
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
        border: 1px solid #8bd5ff;
        border-radius: 4px;
        background: #071018f2;
        color: #d8f3ff;
        box-shadow: 0 4px 24px #000a;
        font: 12px/1.35 ui-monospace, SFMono-Regular, Consolas, monospace;
        white-space: pre-wrap;
      }
      #gpe-web-diagnostics header {
        display: flex;
        gap: 8px;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
      }
      #gpe-web-diagnostics strong { color: #8bd5ff; }
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
      <strong>GPE WEB DIAGNOSTICS</strong>
      <button type="button">COPY</button>
    </header>
    <pre>initializing diagnostics…</pre>
  `;
  document.body.appendChild(root);
  return {
    root,
    output: root.querySelector("pre"),
    copy: root.querySelector("button"),
  };
}

export function installGpeWebDiagnostics() {
  if (!diagnosticsRequested(globalThis.location?.search ?? "")) return null;

  const panel = makePanel();
  const timeline = createBoundedTimeline({ maxEvents: MAX_TIMELINE_EVENTS });
  let snapshotProvider = null;
  let startupState = "page shell initializing";
  let startupError = null;
  let gpuFacts = {
    available: Boolean(navigator.gpu),
    adapter: "probing…",
  };

  function render() {
    panel.output.textContent = report();
  }

  function markEvent(label, detail = null) {
    timeline.mark(label, detail);
    render();
  }

  markEvent("diagnostics installed");
  installWebGpuApiTrace(markEvent);
  markEvent("browser GPU adapter probe started", `navigator.gpu=${Boolean(navigator.gpu)}`);

  browserGpuFacts().then((facts) => {
    gpuFacts = facts;
    markEvent("browser GPU adapter probe finished", facts.adapter);
  });

  function report() {
    const canvas = canvasFacts();
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

    return [
      "GPE WEB DIAGNOSTICS",
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
      "Startup timeline",
      formatTimeline(timeline.snapshot()),
      "",
      "GPE engine observation",
      engine,
      "",
      "Notes",
      "  Browser adapter data is a separate JS probe; it is not claimed to be the adapter selected by GPE/wgpu.",
      "  WebGPU API tracing is diagnostics-only and may slightly perturb timing; use it to locate long waits, not to benchmark absolute latency.",
      "  Unknown data is intentionally left unknown rather than inferred.",
    ].join("\n");
  }

  panel.copy.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(report());
      panel.copy.textContent = "COPIED";
    } catch {
      panel.copy.textContent = "COPY FAILED";
    }
    setTimeout(() => {
      panel.copy.textContent = "COPY";
    }, 1500);
  });

  const timer = setInterval(render, REFRESH_MS);
  globalThis.addEventListener("pagehide", () => clearInterval(timer), { once: true });
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
