import { describe, expect, it } from "vitest";

import {
  createBoundedTimeline,
  diagnosticsRequested,
  errorMessage,
  isWinitControlFlowHandoff,
  safeString,
} from "../../web/diagnostics-core.js";

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
