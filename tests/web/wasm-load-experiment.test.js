import { describe, expect, it } from "vitest";

import {
  initializeArcadeWasm,
  resolveWasmLoadMode,
  WASM_LOAD_ARRAYBUFFER,
  WASM_LOAD_STREAMING,
} from "../../web/wasm-load-experiment.js";

describe("WASM load mode selection", () => {
  it.each([
    ["", WASM_LOAD_STREAMING],
    ["?diagnostics=1", WASM_LOAD_STREAMING],
    ["?diagnostics=1&wasm_load=streaming", WASM_LOAD_STREAMING],
    ["?diagnostics=1&wasm_load=arraybuffer", WASM_LOAD_ARRAYBUFFER],
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
