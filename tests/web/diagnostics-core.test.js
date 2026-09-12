import { describe, expect, it } from "vitest";

import {
  createBoundedTimeline,
  diagnosticsRequested,
  errorMessage,
  formatTimeline,
  isWinitControlFlowHandoff,
  safeString,
  snapshotUnavailableDuringStartup,
  webGpuApiAvailable,
} from "../../web/diagnostics-core.js";
import {
  installWebGpuApiTrace,
  instrumentAdapterRequestDevice,
} from "../../web/diagnostics.js";

describe("diagnosticsRequested", () => {
  it.each([
    ["?diagnostics=1", true],
    ["?diagnostics=true", true],
    ["?diagnostics=on", true],
    ["?diagnostics=0", false],
    ["?diagnostics=false", false],
    ["?foo=1", false],
    ["", false],
  ])("parses %s", (search, expected) => {
    expect(diagnosticsRequested(search)).toBe(expected);
  });
});

describe("webGpuApiAvailable", () => {
  it("accepts a WebGPU object exposing requestAdapter", () => {
    expect(webGpuApiAvailable({ requestAdapter() {} })).toBe(true);
  });

  it.each([undefined, null, {}, { requestAdapter: true }])(
    "rejects missing or invalid WebGPU API: %j",
    (gpu) => {
      expect(webGpuApiAvailable(gpu)).toBe(false);
    },
  );
});

describe("isWinitControlFlowHandoff", () => {
  it("accepts the known winit WASM control-flow sentinel", () => {
    const error = new Error(
      "Using exceptions for control flow, don't mind me. This isn't actually an error!",
    );

    expect(isWinitControlFlowHandoff(error)).toBe(true);
  });

  it("rejects a real startup error", () => {
    expect(isWinitControlFlowHandoff(new Error("requestAdapter returned null"))).toBe(false);
  });

  it("requires both sentinel fragments to avoid broad false positives", () => {
    expect(isWinitControlFlowHandoff(new Error("Using exceptions for control flow"))).toBe(false);
    expect(isWinitControlFlowHandoff(new Error("This isn't actually an error"))).toBe(false);
  });

  it("handles non-Error throw values", () => {
    expect(
      isWinitControlFlowHandoff(
        "Using exceptions for control flow; This isn't actually an error",
      ),
    ).toBe(true);
  });
});

describe("snapshotUnavailableDuringStartup", () => {
  it("only suppresses snapshot read errors while Arcade WASM is initializing", () => {
    expect(snapshotUnavailableDuringStartup("initializing Arcade WASM")).toBe(true);
    expect(snapshotUnavailableDuringStartup("importing Arcade module")).toBe(false);
    expect(snapshotUnavailableDuringStartup("event loop running (winit handoff)")).toBe(false);
    expect(snapshotUnavailableDuringStartup("failed")).toBe(false);
  });
});

describe("WebGPU API tracing", () => {
  it("traces requestAdapter and requestDevice success without changing return values", async () => {
    const events = [];
    const markEvent = (label, detail = null) => events.push([label, detail]);
    const device = { kind: "device" };
    const adapter = {
      async requestDevice(descriptor) {
        expect(descriptor).toEqual({ label: "test-device" });
        return device;
      },
    };
    const gpu = {
      async requestAdapter(options) {
        expect(options).toEqual({ powerPreference: "high-performance" });
        return adapter;
      },
    };

    installWebGpuApiTrace(gpu, markEvent, new WeakSet());
    const returnedAdapter = await gpu.requestAdapter({ powerPreference: "high-performance" });
    const returnedDevice = await returnedAdapter.requestDevice({ label: "test-device" });

    expect(returnedAdapter).toBe(adapter);
    expect(returnedDevice).toBe(device);
    expect(events).toEqual([
      ["WebGPU API trace installed", "requestAdapter/requestDevice"],
      ["WebGPU requestAdapter started", null],
      ["WebGPU requestAdapter resolved", "adapter selected"],
      ["WebGPU requestDevice started", null],
      ["WebGPU requestDevice resolved", null],
    ]);
  });

  it("traces and rethrows requestAdapter rejection", async () => {
    const events = [];
    const failure = new Error("adapter boom");
    const gpu = {
      async requestAdapter() {
        throw failure;
      },
    };

    installWebGpuApiTrace(gpu, (label, detail = null) => events.push([label, detail]), new WeakSet());

    await expect(gpu.requestAdapter()).rejects.toBe(failure);
    expect(events).toContainEqual(["WebGPU requestAdapter rejected", "Error: adapter boom"]);
  });

  it("traces and rethrows requestDevice rejection", async () => {
    const events = [];
    const failure = new Error("device boom");
    const adapter = {
      async requestDevice() {
        throw failure;
      },
    };

    instrumentAdapterRequestDevice(
      adapter,
      (label, detail = null) => events.push([label, detail]),
      new WeakSet(),
    );

    await expect(adapter.requestDevice()).rejects.toBe(failure);
    expect(events).toEqual([
      ["WebGPU requestDevice started", null],
      ["WebGPU requestDevice rejected", "Error: device boom"],
    ]);
  });

  it("reports missing requestAdapter without throwing", () => {
    const events = [];
    installWebGpuApiTrace(
      {},
      (label, detail = null) => events.push([label, detail]),
      new WeakSet(),
    );

    expect(events).toEqual([
      ["WebGPU API trace unavailable", "navigator.gpu.requestAdapter missing"],
    ]);
  });

  it("instruments one adapter only once", async () => {
    const events = [];
    const traced = new WeakSet();
    const adapter = {
      async requestDevice() {
        return "device";
      },
    };
    const markEvent = (label, detail = null) => events.push([label, detail]);

    instrumentAdapterRequestDevice(adapter, markEvent, traced);
    const wrapped = adapter.requestDevice;
    instrumentAdapterRequestDevice(adapter, markEvent, traced);

    expect(adapter.requestDevice).toBe(wrapped);
    await adapter.requestDevice();
    expect(events.filter(([label]) => label === "WebGPU requestDevice started")).toHaveLength(1);
  });
});

describe("createBoundedTimeline", () => {
  it("keeps relative monotonic timestamps and insertion order", () => {
    const values = [100, 101.5, 104];
    const timeline = createBoundedTimeline({
      maxEvents: 4,
      now: () => values.shift(),
    });

    timeline.mark("first");
    timeline.mark("second", "detail");

    expect(timeline.snapshot()).toEqual([
      { ms: 1.5, label: "first", detail: null },
      { ms: 4, label: "second", detail: "detail" },
    ]);
  });

  it("drops the oldest events when the configured bound is exceeded", () => {
    let now = 0;
    const timeline = createBoundedTimeline({
      maxEvents: 2,
      now: () => now++,
    });

    timeline.mark("one");
    timeline.mark("two");
    timeline.mark("three");

    expect(timeline.snapshot().map((event) => event.label)).toEqual(["two", "three"]);
  });

  it("returns defensive snapshots", () => {
    let now = 0;
    const timeline = createBoundedTimeline({ now: () => now++ });
    timeline.mark("stable");

    const snapshot = timeline.snapshot();
    snapshot[0].label = "mutated";

    expect(timeline.snapshot()[0].label).toBe("stable");
  });

  it("rejects invalid bounds", () => {
    expect(() => createBoundedTimeline({ maxEvents: 0 })).toThrow(RangeError);
    expect(() => createBoundedTimeline({ maxEvents: 1.5 })).toThrow(RangeError);
  });
});

describe("formatTimeline", () => {
  it("formats an empty timeline deterministically", () => {
    expect(formatTimeline([])).toBe("  no events");
  });

  it("formats timestamps, labels and optional details exactly", () => {
    expect(
      formatTimeline([
        { ms: 0, label: "diagnostics installed", detail: null },
        { ms: 12.34, label: "adapter selected", detail: "vendor=nvidia" },
      ]),
    ).toBe(
      [
        "  T+    0.0 ms  diagnostics installed",
        "  T+   12.3 ms  adapter selected — vendor=nvidia",
      ].join("\n"),
    );
  });
});

describe("string helpers", () => {
  it("uses explicit fallback only for empty-like values", () => {
    expect(safeString(undefined, "fallback")).toBe("fallback");
    expect(safeString(null, "fallback")).toBe("fallback");
    expect(safeString("", "fallback")).toBe("fallback");
    expect(safeString(false, "fallback")).toBe("false");
    expect(safeString(0, "fallback")).toBe("0");
  });

  it("extracts Error messages and stringifies arbitrary throw values", () => {
    expect(errorMessage(new Error("boom"))).toBe("boom");
    expect(errorMessage("boom")).toBe("boom");
  });
});
