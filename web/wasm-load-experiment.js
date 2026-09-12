import { safeString } from "./diagnostics-core.js";

export const WASM_LOAD_STREAMING = "streaming";
export const WASM_LOAD_ARRAYBUFFER = "arraybuffer";
export const WASM_LOAD_CHUNKED = "chunked";

export function resolveWasmLoadMode(search = globalThis.location?.search ?? "") {
  const requested = new URLSearchParams(search).get("wasm_load");
  if (requested === WASM_LOAD_ARRAYBUFFER) return WASM_LOAD_ARRAYBUFFER;
  if (requested === WASM_LOAD_CHUNKED) return WASM_LOAD_CHUNKED;
  return WASM_LOAD_STREAMING;
}

function normalizeChunk(value) {
  if (value instanceof Uint8Array) return new Uint8Array(value);
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength));
  }
  throw new TypeError("WASM response.body reader returned a non-byte chunk");
}

function formatGapMs(value) {
  return `${Math.round(Math.max(0, Number(value) || 0))} ms`;
}

export async function initializeArcadeWasm({
  init,
  mode = WASM_LOAD_STREAMING,
  wasmUrl = "./pkg/arcade-web_bg.wasm",
  fetchFn = globalThis.fetch?.bind(globalThis),
  markEvent = () => {},
  now = () => globalThis.performance?.now?.() ?? Date.now(),
  slowChunkGapMs = 1000,
} = {}) {
  if (typeof init !== "function") {
    throw new TypeError("init must be a function");
  }

  const selectedMode =
    mode === WASM_LOAD_ARRAYBUFFER
      ? WASM_LOAD_ARRAYBUFFER
      : mode === WASM_LOAD_CHUNKED
        ? WASM_LOAD_CHUNKED
        : WASM_LOAD_STREAMING;
  markEvent("WASM load mode selected", selectedMode);

  if (selectedMode === WASM_LOAD_STREAMING) {
    return init();
  }

  if (typeof fetchFn !== "function") {
    throw new Error("fetch unavailable for WASM loader experiment");
  }

  if (selectedMode === WASM_LOAD_CHUNKED) {
    markEvent("WASM Chunked fetch started", safeString(wasmUrl));
    let response;
    try {
      response = await fetchFn(wasmUrl);
    } catch (error) {
      markEvent("WASM Chunked fetch rejected", safeString(error));
      throw error;
    }

    markEvent(
      "WASM Chunked fetch resolved",
      `status=${safeString(response?.status, "unknown")}`,
    );

    if (response && typeof response.ok === "boolean" && !response.ok) {
      const error = new Error(`WASM fetch failed with HTTP ${safeString(response.status)}`);
      markEvent("WASM Chunked load rejected", error.message);
      throw error;
    }

    if (!response?.body || typeof response.body.getReader !== "function") {
      const error = new TypeError("WASM fetch response does not expose response.body.getReader()");
      markEvent("WASM Chunked load rejected", error.message);
      throw error;
    }

    const reader = response.body.getReader();
    const chunks = [];
    let chunkCount = 0;
    let totalBytes = 0;
    let previousChunkAt = now();
    const slowGapThreshold = Math.max(0, Number(slowChunkGapMs) || 0);

    markEvent("WASM response.body reader started");

    try {
      while (true) {
        let readResult;
        try {
          readResult = await reader.read();
        } catch (error) {
          markEvent("WASM response.body rejected", safeString(error));
          throw error;
        }

        const observedAt = now();
        if (readResult?.done) break;

        let chunk;
        try {
          chunk = normalizeChunk(readResult?.value);
        } catch (error) {
          markEvent("WASM response.body rejected", safeString(error));
          throw error;
        }

        chunkCount += 1;
        const gapMs = Math.max(0, observedAt - previousChunkAt);
        totalBytes += chunk.byteLength;
        chunks.push(chunk);

        if (gapMs >= slowGapThreshold && slowGapThreshold > 0) {
          markEvent(
            "WASM response.body slow gap",
            `index=${chunkCount}; gap=${formatGapMs(gapMs)}`,
          );
        }

        markEvent(
          chunkCount === 1 ? "WASM response.body first chunk" : "WASM response.body chunk",
          `index=${chunkCount}; bytes=${chunk.byteLength}; total=${totalBytes}; gap=${formatGapMs(gapMs)}`,
        );
        previousChunkAt = observedAt;
      }
    } finally {
      if (typeof reader.releaseLock === "function") reader.releaseLock();
    }

    markEvent("WASM response.body completed", `chunks=${chunkCount}; bytes=${totalBytes}`);

    const bytes = new Uint8Array(totalBytes);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.byteLength;
    }

    return init(bytes.buffer);
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
