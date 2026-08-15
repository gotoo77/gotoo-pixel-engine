(() => {
  const contexts = new Set();

  function wrapAudioContext(name) {
    const Original = window[name];
    if (!Original) {
      return;
    }

    window[name] = new Proxy(Original, {
      construct(target, args) {
        const context = Reflect.construct(target, args);
        contexts.add(context);
        return context;
      },
    });
  }

  wrapAudioContext("AudioContext");
  wrapAudioContext("webkitAudioContext");

  function resumeAudio() {
    for (const context of contexts) {
      if (context.state === "closed") {
        contexts.delete(context);
      } else if (context.state === "suspended") {
        context.resume().catch(() => {});
      }
    }
  }

  for (const type of ["pointerup", "touchend", "click", "keydown", "keyup"]) {
    document.addEventListener(type, resumeAudio, { capture: true });
  }
})();
