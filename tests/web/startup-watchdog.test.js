import { describe, expect, it } from "vitest";

import {
  classifyStartupElapsed,
  createStartupWatchdog,
} from "../../web/startup-watchdog.js";

function fakeScheduler() {
  let nextId = 1;
  const callbacks = new Map();
  return {
    schedule(callback) {
      const id = nextId++;
      callbacks.set(id, callback);
      return id;
    },
    cancel(id) {
      callbacks.delete(id);
    },
    runPending() {
      const pending = [...callbacks.entries()];
      callbacks.clear();
      for (const [, callback] of pending) callback();
    },
    pendingCount() {
      return callbacks.size;
    },
  };
}

describe("startup duration classification", () => {
  it.each([
    [999, "FAST"],
    [1000, "SUSPICIOUS"],
    [2999, "SUSPICIOUS"],
    [3000, "SLOW"],
    [7999, "SLOW"],
    [8000, "VERY SLOW"],
  ])("classifies %d ms as %s", (ms, expected) => {
    expect(classifyStartupElapsed(ms)).toBe(expected);
  });
});

describe("startup watchdog", () => {
  it("detects a 3 second stall and records recovery on the next milestone", () => {
    let now = 0;
    const events = [];
    const scheduler = fakeScheduler();
    const watchdog = createStartupWatchdog({
      now: () => now,
      schedule: (callback) => scheduler.schedule(callback),
      cancel: (id) => scheduler.cancel(id),
      emit: (label, detail) => events.push([label, detail]),
    });

    watchdog.observe("Arcade WASM initialization started");
    now = 3100;
    scheduler.runPending();

    expect(events[0]).toEqual([
      "SLOW STARTUP DETECTED",
      "stalled after: Arcade WASM initialization started; 3100 ms without progress",
    ]);
    expect(watchdog.snapshot().status).toBe("SLOW STARTUP DETECTED");

    now = 9400;
    watchdog.observe("canvas attached to DOM");

    expect(events[1]).toEqual([
      "SLOW STARTUP RECOVERED",
      "stall=9400 ms; next=canvas attached to DOM",
    ]);
    expect(watchdog.snapshot()).toMatchObject({
      status: "RECOVERED",
      slowObserved: true,
      lastSlowDurationMs: 9400,
      lastMilestone: "canvas attached to DOM",
    });
  });

  it("completes at requestDevice resolution and does not later report a false stall", () => {
    let now = 0;
    const events = [];
    const scheduler = fakeScheduler();
    const watchdog = createStartupWatchdog({
      now: () => now,
      schedule: (callback) => scheduler.schedule(callback),
      cancel: (id) => scheduler.cancel(id),
      emit: (label, detail) => events.push([label, detail]),
    });

    watchdog.observe("Arcade WASM initialization started");
    now = 400;
    watchdog.observe("WebGPU requestDevice resolved");

    expect(watchdog.snapshot()).toMatchObject({
      status: "complete",
      classification: "FAST",
      complete: true,
      elapsedMs: 400,
    });
    expect(scheduler.pendingCount()).toBe(0);

    now = 5000;
    scheduler.runPending();
    expect(events).toEqual([]);
  });

  it("keeps slow evidence after a recovered run completes", () => {
    let now = 0;
    const scheduler = fakeScheduler();
    const watchdog = createStartupWatchdog({
      now: () => now,
      schedule: (callback) => scheduler.schedule(callback),
      cancel: (id) => scheduler.cancel(id),
    });

    watchdog.observe("WASM instantiateStreaming started");
    now = 3500;
    scheduler.runPending();
    now = 9000;
    watchdog.observe("WASM instantiateStreaming resolved");
    now = 9500;
    watchdog.observe("WebGPU requestDevice resolved");

    expect(watchdog.snapshot()).toMatchObject({
      status: "COMPLETE AFTER SLOW STARTUP",
      classification: "VERY SLOW",
      slowObserved: true,
      lastSlowDurationMs: 9000,
      complete: true,
    });
  });

  it("rejects an invalid threshold", () => {
    expect(() => createStartupWatchdog({ thresholdMs: 0 })).toThrow(RangeError);
  });
});
