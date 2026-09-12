import { describe, expect, it } from "vitest";

import {
  initializeArcadeWasm,
  resolveWasmLoadMode,
  WASM_LOAD_ARRAYBUFFER,
  WASM_LOAD_CHUNKED,
  WASM_LOAD_STREAMING,
} from "../../web/wasm-load-experiment.js";

describe("WASM load mode selection", () => {
  it.each([
    ["", WASM_LOAD_STREAMING],
    ["?diagnostics=1", WASM_LOAD_STREAMING],
    ["?diagnostics=1&wasm_load=streaming", WASM_LOAD_STREAMING],
    ["?diagnostics=1&wasm_load=arraybuffer", WASM_LOAD_ARRAYBUFFER],
    ["?diagnostics=1&wasm_load=chunked", WASM_LOAD_CHUNKED],
    ["?wasm_load=anything-else", WASM_LOAD_STREAMING],
  ])("resolves %s to %s", (search, expected) => {
    expect(resolveWasmLoadMode(search)).toBe(expected);
  });
});

describe("WASM loader experiment", () => {
  it("keeps the normal streaming initializer path unchanged", async () => {
    const events = [];
    const calls = [];
    const result = { ok: true };

    const value = await initializeArcadeWasm({
      init: (...args) => {
        calls.push(args);
        return result;
      },
      mode: WASM_LOAD_STREAMING,
      fetchFn: () => {
        throw new Error("streaming mode must not fetch explicitly");
      },
      markEvent: (label, detail = null) => events.push([label, detail]),
    });

    expect(value).toBe(result);
    expect(calls).toEqual([[]]);
    expect(events).toEqual([["WASM load mode selected", "streaming"]]);
  });

  it("fetches bytes then passes ArrayBuffer to wasm-bindgen init", async () => {
    const events = [];
    const bytes = new ArrayBuffer(8);
    const calls = [];
    const result = { ok: true };

    const value = await initializeArcadeWasm({
      init: async (input) => {
        calls.push(input);
        return result;
      },
      mode: WASM_LOAD_ARRAYBUFFER,
      wasmUrl: "./pkg/arcade-web_bg.wasm",
      fetchFn: async () => ({
        ok: true,
        status: 200,
        async arrayBuffer() {
          return bytes;
        },
      }),
      markEvent: (label, detail = null) => events.push([label, detail]),
    });

    expect(value).toBe(result);
    expect(calls).toEqual([bytes]);
    expect(events).toEqual([
      ["WASM load mode selected", "arraybuffer"],
      ["WASM ArrayBuffer fetch started", "./pkg/arcade-web_bg.wasm"],
      ["WASM ArrayBuffer fetch resolved", "status=200"],
      ["WASM response.arrayBuffer started", null],
      ["WASM response.arrayBuffer resolved", "8 bytes"],
    ]);
  });

  it("reads response.body chunk-by-chunk and traces progress plus long gaps", async () => {
    const events = [];
    const calls = [];
    const result = { ok: true };
    const reads = [
      { done: false, value: new Uint8Array([1, 2]) },
      { done: false, value: new Uint8Array([3]) },
      { done: true, value: undefined },
    ];
    let released = false;
    const times = [100, 150, 1800, 1810];

    const value = await initializeArcadeWasm({
      init: async (input) => {
        calls.push(input);
        return result;
      },
      mode: WASM_LOAD_CHUNKED,
      wasmUrl: "./pkg/arcade-web_bg.wasm",
      fetchFn: async () => ({
        ok: true,
        status: 200,
        body: {
          getReader() {
            return {
              async read() {
                return reads.shift();
              },
              releaseLock() {
                released = true;
              },
            };
          },
        },
      }),
      now: () => times.shift() ?? 1810,
      slowChunkGapMs: 1000,
      markEvent: (label, detail = null) => events.push([label, detail]),
    });

    expect(value).toBe(result);
    expect(released).toBe(true);
    expect(calls).toHaveLength(1);
    expect(new Uint8Array(calls[0])).toEqual(new Uint8Array([1, 2, 3]));
    expect(events).toEqual([
      ["WASM load mode selected", "chunked"],
      ["WASM Chunked fetch started", "./pkg/arcade-web_bg.wasm"],
      ["WASM Chunked fetch resolved", "status=200"],
      ["WASM response.body reader started", null],
      ["WASM response.body first chunk", "index=1; bytes=2; total=2; gap=50 ms"],
      ["WASM response.body slow gap", "index=2; gap=1650 ms"],
      ["WASM response.body chunk", "index=2; bytes=1; total=3; gap=1650 ms"],
      ["WASM response.body completed", "chunks=2; bytes=3"],
    ]);
  });

  it("reports response.body reader failures without calling wasm-bindgen init", async () => {
    const failure = new Error("stream read failed");
    const events = [];
    const calls = [];

    await expect(
      initializeArcadeWasm({
        init: (input) => calls.push(input),
        mode: WASM_LOAD_CHUNKED,
        fetchFn: async () => ({
          ok: true,
          status: 200,
          body: {
            getReader() {
              return {
                async read() {
                  throw failure;
                },
                releaseLock() {},
              };
            },
          },
        }),
        markEvent: (label, detail = null) => events.push([label, detail]),
      }),
    ).rejects.toBe(failure);

    expect(calls).toEqual([]);
    expect(events).toContainEqual([
      "WASM response.body rejected",
      "Error: stream read failed",
    ]);
  });

  it("fails explicitly on a non-success HTTP response", async () => {
    const events = [];
    const initCalls = [];

    await expect(
      initializeArcadeWasm({
        init: (input) => initCalls.push(input),
        mode: WASM_LOAD_ARRAYBUFFER,
        fetchFn: async () => ({ ok: false, status: 503, arrayBuffer: async () => new ArrayBuffer(0) }),
        markEvent: (label, detail = null) => events.push([label, detail]),
      }),
    ).rejects.toThrow("WASM fetch failed with HTTP 503");

    expect(initCalls).toEqual([]);
    expect(events).toContainEqual([
      "WASM ArrayBuffer load rejected",
      "WASM fetch failed with HTTP 503",
    ]);
  });

  it("preserves response.arrayBuffer failures", async () => {
    const failure = new Error("body read failed");
    const events = [];

    await expect(
      initializeArcadeWasm({
        init: () => {},
        mode: WASM_LOAD_ARRAYBUFFER,
        fetchFn: async () => ({
          ok: true,
          status: 200,
          async arrayBuffer() {
            throw failure;
          },
        }),
        markEvent: (label, detail = null) => events.push([label, detail]),
      }),
    ).rejects.toBe(failure);

    expect(events).toContainEqual([
      "WASM response.arrayBuffer rejected",
      "Error: body read failed",
    ]);
  });
});
