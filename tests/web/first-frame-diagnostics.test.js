import { describe, expect, it, vi } from "vitest";

import {
  applyDiagnosticStatus,
  createFirstFrameTiming,
  deriveDiagnosticStatus,
  diagnosticsFilename,
  instrumentGpuQueueSubmit,
  saveDiagnosticsText,
} from "../../web/diagnostics.js";

describe("first-frame timing", () => {
  it("records device readiness, first submit and post-submit RAF with deltas", () => {
    let now = 100;
    const timing = createFirstFrameTiming({ now: () => now });

    now = 180;
    timing.observe("WebGPU requestDevice resolved");
    now = 450;
    timing.observe("first requestAnimationFrame callback");
    now = 470;
    timing.observe("first GPUQueue.submit");
    now = 486;
    timing.observe("first post-submit requestAnimationFrame callback");

    expect(timing.snapshot()).toMatchObject({
      status: "complete",
      classification: "FAST",
      deviceReadyMs: 80,
      firstRafCallbackMs: 350,
      firstSubmitMs: 370,
      postSubmitRafMs: 386,
      deviceToSubmitMs: 290,
      submitToPostRafMs: 16,
      complete: true,
    });
  });

  it("reports a slow first-frame wait after the device is ready", () => {
    let now = 0;
    const timing = createFirstFrameTiming({ now: () => now, slowThresholdMs: 3000 });

    now = 500;
    timing.observe("WebGPU requestDevice resolved");
    now = 4200;

    expect(timing.snapshot()).toMatchObject({
      status: "SLOW FIRST FRAME",
      classification: "SLOW",
      waitingFor: "first GPUQueue.submit",
      deviceReadyMs: 500,
      firstSubmitMs: null,
      stalledForMs: 3700,
      complete: false,
    });
  });

  it("does not report a false slow frame when queue-submit tracing is unavailable", () => {
    let now = 0;
    const timing = createFirstFrameTiming({ now: () => now, slowThresholdMs: 3000 });

    now = 500;
    timing.observe("WebGPU requestDevice resolved");
    now = 550;
    timing.observe("GPUQueue.submit trace unavailable", "TypeError: read only");
    now = 8000;

    expect(timing.snapshot()).toMatchObject({
      status: "UNAVAILABLE",
      classification: "N/A",
      waitingFor: null,
      stalledForMs: 0,
      unavailableReason: "GPUQueue.submit trace unavailable: TypeError: read only",
      complete: false,
    });
  });

  it("does not report a false slow frame when post-submit RAF is unavailable", () => {
    let now = 0;
    const timing = createFirstFrameTiming({ now: () => now, slowThresholdMs: 3000 });

    now = 500;
    timing.observe("WebGPU requestDevice resolved");
    now = 700;
    timing.observe("first GPUQueue.submit");
    now = 710;
    timing.observe("first post-submit requestAnimationFrame unavailable", "requestAnimationFrame missing");
    now = 8000;

    expect(timing.snapshot()).toMatchObject({
      status: "UNAVAILABLE",
      classification: "N/A",
      waitingFor: null,
      stalledForMs: 0,
      unavailableReason:
        "first post-submit requestAnimationFrame unavailable: requestAnimationFrame missing",
      complete: false,
    });
  });
});

describe("GPU queue submit tracing", () => {
  it("preserves submit arguments and return value while tracing only the first submit", () => {
    const events = [];
    const rafCallbacks = [];
    const calls = [];
    const expected = { returned: true };
    const queue = {
      submit(commandBuffers) {
        calls.push(commandBuffers);
        return expected;
      },
    };

    instrumentGpuQueueSubmit(queue, (label, detail = null) => events.push([label, detail]), {
      traced: new WeakSet(),
      requestAnimationFrameFn: (callback) => {
        rafCallbacks.push(callback);
        return 17;
      },
    });

    const firstBuffers = [{ id: 1 }];
    const secondBuffers = [{ id: 2 }];
    expect(queue.submit(firstBuffers)).toBe(expected);
    expect(queue.submit(secondBuffers)).toBe(expected);
    expect(calls).toEqual([firstBuffers, secondBuffers]);

    expect(events).toEqual([
      ["first GPUQueue.submit", null],
      ["first post-submit requestAnimationFrame scheduled", null],
    ]);
    expect(rafCallbacks).toHaveLength(1);

    rafCallbacks[0]();
    expect(events.at(-1)).toEqual(["first post-submit requestAnimationFrame callback", null]);
  });

  it("is idempotent for the same queue", () => {
    const events = [];
    const traced = new WeakSet();
    const queue = { submit() {} };
    const markEvent = (label, detail = null) => events.push([label, detail]);
    const requestAnimationFrameFn = vi.fn(() => 1);

    instrumentGpuQueueSubmit(queue, markEvent, { traced, requestAnimationFrameFn });
    const wrapped = queue.submit;
    instrumentGpuQueueSubmit(queue, markEvent, { traced, requestAnimationFrameFn });

    expect(queue.submit).toBe(wrapped);
    queue.submit([]);
    expect(events.filter(([label]) => label === "first GPUQueue.submit")).toHaveLength(1);
    expect(requestAnimationFrameFn).toHaveBeenCalledTimes(1);
  });
});

describe("diagnostic severity", () => {
  it("uses ERROR for terminal failures", () => {
    expect(
      deriveDiagnosticStatus({
        startupError: "boom",
        watchdog: { outcome: "failed", status: "FAILED", classification: "N/A" },
        firstFrame: { complete: false, classification: "FAST", status: "monitoring" },
      }),
    ).toEqual({ level: "error", label: "ERROR", marker: "[ERROR]", reason: "startup failed" });
  });

  it("uses WARN for slow startup or first-frame timing", () => {
    expect(
      deriveDiagnosticStatus({
        startupError: null,
        watchdog: { outcome: "success", status: "complete", classification: "FAST" },
        firstFrame: { complete: true, classification: "SUSPICIOUS", status: "complete" },
      }).level,
    ).toBe("warn");
  });

  it("uses WARN when first-frame tracing is unavailable", () => {
    expect(
      deriveDiagnosticStatus({
        startupError: null,
        watchdog: { outcome: "success", status: "complete", classification: "FAST" },
        firstFrame: { complete: false, classification: "N/A", status: "UNAVAILABLE" },
      }),
    ).toEqual({
      level: "warn",
      label: "ATTENTION",
      marker: "[WARN]",
      reason: "first-frame trace unavailable",
    });
  });

  it("uses OK only after a fast complete startup and first frame", () => {
    expect(
      deriveDiagnosticStatus({
        startupError: null,
        watchdog: { outcome: "success", status: "complete", classification: "FAST" },
        firstFrame: { complete: true, classification: "FAST", status: "complete" },
      }),
    ).toEqual({ level: "ok", label: "OK", marker: "[OK]", reason: "startup and first frame complete" });
  });

  it("applies a textual badge and semantic data attribute without relying on color alone", () => {
    const panel = {
      root: { dataset: {} },
      statusBadge: { textContent: "" },
    };
    applyDiagnosticStatus(panel, { level: "warn", label: "ATTENTION", marker: "[WARN]" });

    expect(panel.root.dataset.severity).toBe("warn");
    expect(panel.statusBadge.textContent).toBe("[WARN] ATTENTION");
  });
});

describe("diagnostics text export", () => {
  it("uses a deterministic timestamped .txt filename", () => {
    expect(diagnosticsFilename(new Date("2026-09-12T21:48:12.000Z"))).toBe(
      "gpe-web-diagnostics-20260912-214812.txt",
    );
  });

  it("saves the exact report text through a temporary object URL and revokes it", async () => {
    const clicked = [];
    const removed = [];
    const anchors = [];
    const documentRef = {
      createElement(tag) {
        expect(tag).toBe("a");
        const anchor = {
          href: "",
          download: "",
          style: {},
          click: () => clicked.push(true),
          remove: () => removed.push(true),
        };
        anchors.push(anchor);
        return anchor;
      },
      body: { appendChild() {} },
    };
    const created = [];
    const revoked = [];
    const urlApi = {
      createObjectURL(blob) {
        created.push(blob);
        return "blob:gpe-test";
      },
      revokeObjectURL(url) {
        revoked.push(url);
      },
    };
    const report = "EXACT REPORT\n[OK] done";

    const filename = saveDiagnosticsText(report, {
      documentRef,
      urlApi,
      BlobCtor: Blob,
      date: new Date("2026-09-12T21:48:12.000Z"),
    });

    expect(filename).toBe("gpe-web-diagnostics-20260912-214812.txt");
    expect(anchors[0].href).toBe("blob:gpe-test");
    expect(anchors[0].download).toBe(filename);
    expect(clicked).toEqual([true]);
    expect(removed).toEqual([true]);
    expect(revoked).toEqual(["blob:gpe-test"]);
    expect(created).toHaveLength(1);
    expect(created[0].type).toBe("text/plain;charset=utf-8");
    expect(await created[0].text()).toBe(report);
  });
});
