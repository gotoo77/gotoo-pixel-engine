export const DEFAULT_SLOW_STARTUP_THRESHOLD_MS = 3000;

export function classifyStartupElapsed(ms) {
  if (ms < 1000) return "FAST";
  if (ms < 3000) return "SUSPICIOUS";
  if (ms < 8000) return "SLOW";
  return "VERY SLOW";
}

export function createStartupWatchdog({
  thresholdMs = DEFAULT_SLOW_STARTUP_THRESHOLD_MS,
  now = () => performance.now(),
  schedule = (callback, delay) => globalThis.setTimeout(callback, delay),
  cancel = (id) => globalThis.clearTimeout(id),
  emit = () => {},
  terminalLabels = ["WebGPU requestDevice resolved", "startup error"],
} = {}) {
  if (!Number.isFinite(thresholdMs) || thresholdMs <= 0) {
    throw new RangeError("thresholdMs must be a positive finite number");
  }

  const terminal = new Set(terminalLabels);
  const startedAt = now();
  let lastMilestone = "watchdog installed";
  let lastMilestoneAt = startedAt;
  let timer = null;
  let completedAt = null;
  let activeSlow = false;
  let slowObserved = false;
  let lastSlowDurationMs = null;

  function clearTimer() {
    if (timer !== null) {
      cancel(timer);
      timer = null;
    }
  }

  function arm() {
    clearTimer();
    if (completedAt !== null || activeSlow) return;

    timer = schedule(() => {
      timer = null;
      if (completedAt !== null || activeSlow) return;

      const stalledMs = Math.max(0, now() - lastMilestoneAt);
      if (stalledMs < thresholdMs) {
        arm();
        return;
      }

      activeSlow = true;
      slowObserved = true;
      emit(
        "SLOW STARTUP DETECTED",
        `stalled after: ${lastMilestone}; ${Math.round(stalledMs)} ms without progress`,
      );
    }, thresholdMs);
  }

  function observe(label) {
    if (completedAt !== null) return;

    const current = String(label);
    const observedAt = now();

    if (activeSlow) {
      const stallDurationMs = Math.max(0, observedAt - lastMilestoneAt);
      activeSlow = false;
      lastSlowDurationMs = stallDurationMs;
      emit(
        "SLOW STARTUP RECOVERED",
        `stall=${Math.round(stallDurationMs)} ms; next=${current}`,
      );
    }

    lastMilestone = current;
    lastMilestoneAt = observedAt;

    if (terminal.has(current)) {
      completedAt = observedAt;
      clearTimer();
      return;
    }

    arm();
  }

  function snapshot() {
    const sampledAt = completedAt ?? now();
    const elapsedMs = Math.max(0, sampledAt - startedAt);
    const stalledForMs = completedAt === null ? Math.max(0, now() - lastMilestoneAt) : 0;
    let status = "monitoring";
    if (activeSlow) status = "SLOW STARTUP DETECTED";
    else if (completedAt !== null && slowObserved) status = "COMPLETE AFTER SLOW STARTUP";
    else if (completedAt !== null) status = "complete";
    else if (slowObserved) status = "RECOVERED";

    return {
      status,
      classification: classifyStartupElapsed(elapsedMs),
      thresholdMs,
      elapsedMs,
      lastMilestone,
      stalledForMs,
      slowObserved,
      lastSlowDurationMs,
      complete: completedAt !== null,
    };
  }

  function dispose() {
    clearTimer();
  }

  return { observe, snapshot, dispose };
}
