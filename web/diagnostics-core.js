export const WINIT_CONTROL_FLOW_MARKER = "Using exceptions for control flow";
export const WINIT_NOT_ERROR_MARKER = "This isn't actually an error";

export function diagnosticsRequested(search = globalThis.location?.search ?? "") {
  const value = new URLSearchParams(search).get("diagnostics");
  return value === "1" || value === "true" || value === "on";
}

export function webGpuApiAvailable(gpu) {
  return Boolean(gpu && typeof gpu.requestAdapter === "function");
}

export function safeString(value, fallback = "unknown") {
  if (value === undefined || value === null || value === "") return fallback;
  return String(value);
}

export function errorMessage(error) {
  if (typeof error?.message === "string") return error.message;
  return String(error);
}

export function isWinitControlFlowHandoff(error) {
  const message = errorMessage(error);
  return (
    message.includes(WINIT_CONTROL_FLOW_MARKER) &&
    message.includes(WINIT_NOT_ERROR_MARKER)
  );
}

export function snapshotUnavailableDuringStartup(startupState) {
  return startupState === "initializing Arcade WASM";
}

export function createBoundedTimeline({ maxEvents = 64, now = () => performance.now() } = {}) {
  if (!Number.isInteger(maxEvents) || maxEvents < 1) {
    throw new RangeError("maxEvents must be a positive integer");
  }

  const startedAt = now();
  const events = [];

  return {
    mark(label, detail = null) {
      events.push({
        ms: Math.max(0, now() - startedAt),
        label: safeString(label),
        detail: detail === null ? null : safeString(detail),
      });
      while (events.length > maxEvents) events.shift();
      return events.at(-1);
    },
    snapshot() {
      return events.map((event) => ({ ...event }));
    },
  };
}

export function formatTimeline(events) {
  if (!events.length) return "  no events";
  return events
    .map(({ ms, label, detail }) => {
      const suffix = detail ? ` — ${detail}` : "";
      return `  T+${Number(ms).toFixed(1).padStart(7)} ms  ${safeString(label)}${suffix}`;
    })
    .join("\n");
}
