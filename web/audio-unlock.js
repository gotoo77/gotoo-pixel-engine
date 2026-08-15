(() => {
  const contexts = new Set();
  const contextMeta = new Map();
  const activeMedia = new Set();
  const debugEnabled = new URLSearchParams(window.location.search).has("audio-debug");
  const debugState = {
    lastGesture: "none",
    resumeAttempts: 0,
    resumeSuccesses: 0,
    resumeFailures: 0,
    lastError: "none",
    testBeep: "not run",
    mediaBeep: "not run",
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

  function finiteOrUnknown(value) {
    return Number.isFinite(value) ? value : "?";
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
        const sampleRate = finiteOrUnknown(context.sampleRate);
        const baseLatency = finiteOrUnknown(context.baseLatency);
        const outputLatency = finiteOrUnknown(context.outputLatency);
        const channels = finiteOrUnknown(context.destination?.channelCount);
        const maxChannels = finiteOrUnknown(context.destination?.maxChannelCount);
        return `#${id} ${name}: state=${context.state} rate=${sampleRate} base=${baseLatency} out=${outputLatency} ch=${channels}/${maxChannels}`;
      })
      .join("\n");
  }

  function debugReport() {
    const brands = navigator.userAgentData?.brands
      ?.map((brand) => `${brand.brand}/${brand.version}`)
      .join(", ") ?? "n/a";
    const platform = navigator.userAgentData?.platform ?? navigator.platform ?? "n/a";

    return [
      "GPE WebAudio diagnostics",
      `Brave: ${debugState.brave}`,
      `UA brands: ${brands}`,
      `platform: ${platform}; touchPoints=${navigator.maxTouchPoints}`,
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
      `WebAudio loud beep: ${debugState.testBeep}`,
      `<audio> WAV beep: ${debugState.mediaBeep}`,
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
      oscillator.type = "triangle";
      oscillator.frequency.setValueAtTime(440, now);
      oscillator.frequency.setValueAtTime(880, now + 0.3);
      oscillator.frequency.setValueAtTime(660, now + 0.6);
      gain.gain.setValueAtTime(0.0001, now);
      gain.gain.linearRampToValueAtTime(0.35, now + 0.02);
      gain.gain.setValueAtTime(0.35, now + 0.85);
      gain.gain.linearRampToValueAtTime(0.0001, now + 0.95);
      oscillator.connect(gain);
      gain.connect(context.destination);
      oscillator.start(now);
      oscillator.stop(now + 1.0);
      debugState.testBeep = `scheduled LOUD; context=${context.state}`;
      renderDebug();
    } catch (error) {
      debugState.testBeep = "failed";
      setError(error);
    }
  }

  function createToneWav() {
    const sampleRate = 48000;
    const durationSeconds = 0.9;
    const sampleCount = Math.floor(sampleRate * durationSeconds);
    const dataSize = sampleCount * 2;
    const buffer = new ArrayBuffer(44 + dataSize);
    const view = new DataView(buffer);

    function ascii(offset, text) {
      for (let i = 0; i < text.length; i += 1) {
        view.setUint8(offset + i, text.charCodeAt(i));
      }
    }

    ascii(0, "RIFF");
    view.setUint32(4, 36 + dataSize, true);
    ascii(8, "WAVE");
    ascii(12, "fmt ");
    view.setUint32(16, 16, true);
    view.setUint16(20, 1, true);
    view.setUint16(22, 1, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, sampleRate * 2, true);
    view.setUint16(32, 2, true);
    view.setUint16(34, 16, true);
    ascii(36, "data");
    view.setUint32(40, dataSize, true);

    for (let i = 0; i < sampleCount; i += 1) {
      const t = i / sampleRate;
      const frequency = t < 0.3 ? 523.25 : t < 0.6 ? 783.99 : 659.25;
      const attack = Math.min(1, t / 0.02);
      const release = Math.min(1, (durationSeconds - t) / 0.05);
      const envelope = Math.max(0, Math.min(attack, release));
      const sample = Math.sin(2 * Math.PI * frequency * t) * 0.45 * envelope;
      view.setInt16(44 + i * 2, Math.round(sample * 32767), true);
    }

    return new Blob([buffer], { type: "audio/wav" });
  }

  async function playMediaBeep() {
    let url = null;
    try {
      url = URL.createObjectURL(createToneWav());
      const audio = new Audio(url);
      audio.volume = 1;
      audio.preload = "auto";
      audio.playsInline = true;
      activeMedia.add(audio);

      audio.addEventListener(
        "playing",
        () => {
          debugState.mediaBeep = `playing; paused=${audio.paused} volume=${audio.volume} muted=${audio.muted}`;
          renderDebug();
        },
        { once: true },
      );
      audio.addEventListener(
        "ended",
        () => {
          debugState.mediaBeep = "ended normally";
          activeMedia.delete(audio);
          URL.revokeObjectURL(url);
          renderDebug();
        },
        { once: true },
      );
      audio.addEventListener(
        "error",
        () => {
          const code = audio.error?.code ?? "?";
          debugState.mediaBeep = `media error code=${code}`;
          activeMedia.delete(audio);
          URL.revokeObjectURL(url);
          renderDebug();
        },
        { once: true },
      );

      await audio.play();
      debugState.mediaBeep = `play() resolved; paused=${audio.paused} volume=${audio.volume} muted=${audio.muted}`;
      renderDebug();
    } catch (error) {
      debugState.mediaBeep = "failed";
      if (url) {
        URL.revokeObjectURL(url);
      }
      setError(error);
    }
  }

  function makeButton(label, handler) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = label;
    button.style.cssText = "padding:7px;font:12px monospace";
    button.addEventListener("click", handler);
    return button;
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
      "max-height:58vh",
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
    controls.style.cssText = "display:flex;flex-wrap:wrap;gap:8px;margin-bottom:8px;position:sticky;top:0";

    const webAudioButton = makeButton("LOUD WEB AUDIO", playTestBeep);
    const mediaButton = makeButton("WAV <audio>", playMediaBeep);
    const copyButton = makeButton("COPY INFO", async () => {
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

    controls.append(webAudioButton, mediaButton, copyButton);
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
