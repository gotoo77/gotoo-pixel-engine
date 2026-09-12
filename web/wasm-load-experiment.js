import { safeString } from "./diagnostics-core.js";

export const WASM_LOAD_STREAMING = "streaming";
export const WASM_LOAD_ARRAYBUFFER = "arraybuffer";

export function resolveWasmLoadMode(search = globalThis.location?.search ?? "") {
  const requested = new URLSearchParams(search).get("wasm_load");
  return requested === WASM_LOAD_ARRAYBUFFER ? WASM_LOAD_ARRAYBUFFER : WASM_LOAD_STREAMING;
}

export async function initializeArcadeWasm({
  init,
  mode = WASM_LOAD_STREAMING,
  wasmUrl = "./pkg/arcade-web_bg.wasm",
  fetchFn = globalThis.fetch?.bind(globalThis),
  markEvent = () => {},
} = {}) {
  if (typeof init !== "function") {
    throw new TypeError("init must be a function");
  }

  const selectedMode = mode === WASM_LOAD_ARRAYBUFFER ? WASM_LOAD_ARRAYBUFFER : WASM_LOAD_STREAMING;
  markEvent("WASM load mode selected", selectedMode);

  if (selectedMode === WASM_LOAD_STREAMING) {
    return init();
  }

  if (typeof fetchFn !== "function") {
    throw new Error("fetch unavailable for ArrayBuffer WASM loader experiment");
  }

  markEvent("WASM ArrayBuffer fetch started", safeString(wasmUrl));
  let response;
  try {
    response = await fetchFn(wasmUrl);
  } catch (error) {
    markEvent("WASM ArrayBuffer fetch rejected", safeString(error));
    throw error;
  }

  markEvent(
    "WASM ArrayBuffer fetch resolved",
    `status=${safeString(response?.status, "unknown")}`,
  );

  if (response && typeof response.ok === "boolean" && !response.ok) {
    const error = new Error(`WASM fetch failed with HTTP ${safeString(response.status)}`);
    markEvent("WASM ArrayBuffer load rejected", error.message);
    throw error;
  }

  if (!response || typeof response.arrayBuffer !== "function") {
    const error = new TypeError("WASM fetch response does not expose arrayBuffer()");
    markEvent("WASM ArrayBuffer load rejected", error.message);
    throw error;
  }

  markEvent("WASM response.arrayBuffer started");
  let bytes;
  try {
    bytes = await response.arrayBuffer();
  } catch (error) {
    markEvent("WASM response.arrayBuffer rejected", safeString(error));
    throw error;
  }

  const byteLength = Number(bytes?.byteLength);
  markEvent(
    "WASM response.arrayBuffer resolved",
    Number.isFinite(byteLength) ? `${byteLength} bytes` : "byte length unknown",
  );

  return init(bytes);
}
