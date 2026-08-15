(() => {
  const contexts = new Set();
  const contextMeta = new Map();
  const debugEnabled = new URLSearchParams(window.location.search).has("audio-debug");
  const debugState = {
    lastGesture: "none",
    resumeAttempts: 0,
    resumeSuccesses: 0,
    resumeFailures: 0,
    lastError: "none",
    testBeep: "not run",
    brave: "unknown",
  };
  let debugPanel = null;
  let debugText = null;
  let nextContextId = 1;

  function activationSummary() {
    const activation = navigator.userActivation;
    if (!activation) {
      return "unsupported";
    }
    return `active=${activation.isActive} ever=${activation.hasBeenActive}`;
  }

  function contextSummary() {
    if (contexts.size === 0) {
      return "none";
    }

    return [...contexts]
      .map((context) => {
        const meta = contextMeta.get(context);
        const id = meta?.id ?? "?";
        const name = meta?.name ?? "AudioContext";
        const sampleRate = Number.isFinite(context.sampleRate) ? context.sampleRate : "?";
        return `#${id} ${name}: state=${context.state} rate=${sampleRate}`;
      })
      .join("\n");
  }

  function debugReport() {
    const brands = navigator.userAgentData?.brands
      ?.map((brand) => `${brand.brand}/${brand.version}`)
      .join(", ") ?? "n/a";

    return [
      "GPE WebAudio diagnostics",
      `Brave: ${debugState.brave}`,
      `UA brands: ${brands}`,
      `UA: ${navigator.userAgent}`,
      `visibility: ${document.visibilityState}`,
      `fullscreen: ${Boolean(document.fullscreenElement)}`,
      `userActivation: ${activationSummary()}`,
      `AudioContext: ${typeof window.AudioContext}`,
      `webkitAudioContext: ${typeof window.webkitAudioContext}`,
      `contexts (${contexts.size}):`,
      contextSummary(),
      `last gesture: ${debugState.lastGesture}`,
      `resume: ${debugState.resumeSuccesses}/${debugState.resumeAttempts} ok, ${debugState.resumeFailures} failed`,
      `last error: ${debugState.lastError}`,
      `test beep: ${debugState.testBeep}`,
    ].join("\n");
  }

  function renderDebug() {
    if (debugText) {
      debugText.textContent = debugReport();
    }
  }

  function setError(error) {
    debugState.lastError = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
    renderDebug();
  }

  function trackContext(context, name) {
    contexts.add(context);
    contextMeta.set(context, { id: nextContextId++, name });
    context.addEventListener?.("statechange", renderDebug);
    renderDebug();
    return context;
  }

  function wrapAudioContext(name) {
    const Original = window[name];
    if (!Original) {
      return;
    }

    window[name] = new Proxy(Original, {
      construct(target, args, newTarget) {
        const context = Reflect.construct(target, args, newTarget);
        return trackContext(context, name);
      },
    });
  }

  wrapAudioContext("AudioContext");
  wrapAudioContext("webkitAudioContext");

  function resumeAudio(event) {
    debugState.lastGesture = `${event.type}${event.pointerType ? `/${event.pointerType}` : ""}`;
    renderDebug();

    for (const context of contexts) {
      if (context.state === "closed") {
        contexts.delete(context);
        contextMeta.delete(context);
      } else if (context.state === "suspended") {
        debugState.resumeAttempts += 1;
        context
          .resume()
          .then(() => {
            debugState.resumeSuccesses += 1;
            renderDebug();
          })
          .catch((error) => {
            debugState.resumeFailures += 1;
            setError(error);
          });
      }
    }
  }

  async function playTestBeep() {
    try {
      const Context = window.AudioContext || window.webkitAudioContext;
      if (!Context) {
        throw new Error("AudioContext is unavailable");
      }

      const context = new Context();
      if (context.state === "suspended") {
        debugState.resumeAttempts += 1;
        await context.resume();
        debugState.resumeSuccesses += 1;
      }

      const oscillator = context.createOscillator();
      const gain = context.createGain();
      const now = context.currentTime;
      oscillator.frequency.value = 440;
      gain.gain.setValueAtTime(0.0001, now);
      gain.gain.exponentialRampToValueAtTime(0.08, now + 0.01);
      gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.18);
      oscillator.connect(gain);
      gain.connect(context.destination);
      oscillator.start(now);
      oscillator.stop(now + 0.2);
      debugState.testBeep = `played; context=${context.state}`;
      renderDebug();
    } catch (error) {
      debugState.testBeep = "failed";
      setError(error);
    }
  }

  function installDebugPanel() {
    if (!debugEnabled || debugPanel) {
      return;
    }

    const panel = document.createElement("section");
    panel.id = "gpe-audio-debug";
    panel.style.cssText = [
      "position:fixed",
      "left:8px",
      "right:8px",
      "bottom:8px",
      "z-index:2147483647",
      "max-height:52vh",
      "overflow:auto",
      "padding:10px",
      "border:1px solid #78ebb4",
      "background:#08110ff2",
      "color:#d8ffe9",
      "font:11px/1.35 monospace",
      "text-align:left",
      "white-space:pre-wrap",
      "box-sizing:border-box",
    ].join(";");

    const controls = document.createElement("div");
    controls.style.cssText = "display:flex;gap:8px;margin-bottom:8px;position:sticky;top:0";

    const beepButton = document.createElement("button");
    beepButton.type = "button";
    beepButton.textContent = "TEST BEEP";
    beepButton.style.cssText = "padding:7px;font:12px monospace";
    beepButton.addEventListener("click", playTestBeep);

    const copyButton = document.createElement("button");
    copyButton.type = "button";
    copyButton.textContent = "COPY INFO";
    copyButton.style.cssText = "padding:7px;font:12px monospace";
    copyButton.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(debugReport());
        copyButton.textContent = "COPIED";
      } catch (error) {
        setError(error);
        copyButton.textContent = "COPY FAILED";
      }
    });

    const text = document.createElement("pre");
    text.style.cssText = "margin:0;white-space:pre-wrap";

    controls.append(beepButton, copyButton);
    panel.append(controls, text);
    document.documentElement.append(panel);
    debugPanel = panel;
    debugText = text;
    renderDebug();

    if (navigator.brave?.isBrave) {
      navigator.brave
        .isBrave()
        .then((isBrave) => {
          debugState.brave = String(isBrave);
          renderDebug();
        })
        .catch(setError);
    } else {
      debugState.brave = "API unavailable";
      renderDebug();
    }
  }

  for (const type of ["pointerup", "touchend", "click", "keydown", "keyup"]) {
    document.addEventListener(type, resumeAudio, { capture: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installDebugPanel, { once: true });
  } else {
    installDebugPanel();
  }
})();
