import { describe, expect, it } from "vitest";

import {
  installCanvasAttachmentTrace,
  installFirstAnimationFrameTrace,
  installWasmApiTrace,
} from "../../web/startup-trace.js";

describe("WASM startup API tracing", () => {
  it("traces wasm fetch and instantiation without changing return values", async () => {
    const events = [];
    const response = { status: 200 };
    const streamingResult = { kind: "streaming" };
    const instantiateResult = { kind: "instantiate" };
    const scope = {
      async fetch(input) {
        return input.endsWith(".wasm") ? response : { status: 204 };
      },
      WebAssembly: {
        async instantiateStreaming() {
          return streamingResult;
        },
        async instantiate() {
          return instantiateResult;
        },
      },
    };

    installWasmApiTrace(scope, (label, detail = null) => events.push([label, detail]), new WeakSet());

    expect(await scope.fetch("./pkg/arcade-web_bg.wasm")).toBe(response);
    expect(await scope.fetch("./favicon.ico")).toEqual({ status: 204 });
    expect(await scope.WebAssembly.instantiateStreaming(Promise.resolve(response), {})).toBe(
      streamingResult,
    );
    expect(await scope.WebAssembly.instantiate(new Uint8Array([0, 97, 115, 109]), {})).toBe(
      instantiateResult,
    );

    expect(events).toEqual([
      ["WASM startup API trace installed", "fetch/instantiateStreaming/instantiate"],
      ["WASM fetch started", "arcade-web_bg.wasm"],
      ["WASM fetch resolved", "arcade-web_bg.wasm; status=200"],
      ["WASM instantiateStreaming started", null],
      ["WASM instantiateStreaming resolved", null],
      ["WASM instantiate started", null],
      ["WASM instantiate resolved", null],
    ]);
  });

  it("traces failures and rethrows the original error", async () => {
    const events = [];
    const failure = new Error("compile boom");
    const scope = {
      async fetch() {
        return { status: 200 };
      },
      WebAssembly: {
        async instantiateStreaming() {
          throw failure;
        },
      },
    };

    installWasmApiTrace(scope, (label, detail = null) => events.push([label, detail]), new WeakSet());

    await expect(scope.WebAssembly.instantiateStreaming({}, {})).rejects.toBe(failure);
    expect(events).toContainEqual([
      "WASM instantiateStreaming rejected",
      "Error: compile boom",
    ]);
  });

  it("is idempotent for one scope", async () => {
    const events = [];
    const traced = new WeakSet();
    const scope = {
      async fetch() {
        return { status: 200 };
      },
      WebAssembly: {},
    };
    const markEvent = (label, detail = null) => events.push([label, detail]);

    installWasmApiTrace(scope, markEvent, traced);
    const wrappedFetch = scope.fetch;
    installWasmApiTrace(scope, markEvent, traced);

    expect(scope.fetch).toBe(wrappedFetch);
    await scope.fetch("game.wasm");
    expect(events.filter(([label]) => label === "WASM fetch started")).toHaveLength(1);
  });
});

describe("winit browser-visible startup tracing", () => {
  it("records the first animation frame schedule and callback once", () => {
    const events = [];
    const callbacks = [];
    const scope = {
      requestAnimationFrame(callback) {
        callbacks.push(callback);
        return callbacks.length;
      },
    };

    installFirstAnimationFrameTrace(
      scope,
      (label, detail = null) => events.push([label, detail]),
      new WeakSet(),
    );

    expect(scope.requestAnimationFrame(() => {})).toBe(1);
    expect(scope.requestAnimationFrame(() => {})).toBe(2);
    callbacks[0](123);
    callbacks[1](124);

    expect(events).toEqual([
      ["animation frame trace installed", null],
      ["first requestAnimationFrame scheduled", null],
      ["first requestAnimationFrame callback", null],
    ]);
  });

  it("records when a canvas is attached and then disconnects", () => {
    const events = [];
    let observerCallback = null;
    let disconnected = false;
    const observed = [];

    class FakeMutationObserver {
      constructor(callback) {
        observerCallback = callback;
      }

      observe(target, options) {
        observed.push([target, options]);
      }

      disconnect() {
        disconnected = true;
      }
    }

    const root = { kind: "root" };
    const documentObject = {
      documentElement: root,
      querySelector() {
        return null;
      },
    };

    installCanvasAttachmentTrace(
      documentObject,
      FakeMutationObserver,
      (label, detail = null) => events.push([label, detail]),
      new WeakSet(),
    );

    expect(observed).toEqual([[root, { childList: true, subtree: true }]]);
    observerCallback([{ addedNodes: [{ tagName: "CANVAS" }] }]);

    expect(events).toEqual([
      ["canvas DOM trace installed", null],
      ["canvas attached to DOM", null],
    ]);
    expect(disconnected).toBe(true);
  });
});
