const STYLE_ID = "gpe-fullscreen-style";
const BUTTON_ID = "gpe-fullscreen-button";
const FAVICON_ID = "gpe-favicon";
let touchFullscreenAttempted = false;

function installBranding() {
  if (document.getElementById(FAVICON_ID)) {
    return;
  }

  const favicon = document.createElement("link");
  favicon.id = FAVICON_ID;
  favicon.rel = "icon";
  favicon.type = "image/svg+xml";
  favicon.href = "./favicon.svg";
  document.head.append(favicon);
}

function installStyle() {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    #${BUTTON_ID} {
      position: fixed;
      top: 10px;
      left: 10px;
      z-index: 1000;
      padding: 6px 9px;
      border: 1px solid #78ebb4;
      background: #111d;
      color: #78ebb4;
      font: 12px monospace;
      cursor: pointer;
      touch-action: manipulation;
    }

    :fullscreen canvas {
      width: 100vw !important;
      height: 100vh !important;
      max-width: none !important;
      max-height: none !important;
    }
  `;
  document.head.append(style);
}

function updateButton(button) {
  button.textContent = document.fullscreenElement ? "EXIT FULLSCREEN" : "FULLSCREEN";
}

async function toggleFullscreen() {
  try {
    if (document.fullscreenElement) {
      await document.exitFullscreen();
    } else {
      await document.documentElement.requestFullscreen();
    }
  } catch (error) {
    console.warn("GPE fullscreen request failed", error);
  }
}

async function enterFullscreenFromTouch(event) {
  if (
    touchFullscreenAttempted ||
    document.fullscreenElement ||
    !document.fullscreenEnabled ||
    (event.pointerType && event.pointerType !== "touch")
  ) {
    return;
  }

  touchFullscreenAttempted = true;
  try {
    await document.documentElement.requestFullscreen();
  } catch (_) {
    // Fullscreen is opportunistic on touch devices; keep normal play if denied.
  }
}

function installTouchFullscreen() {
  document.addEventListener("pointerup", enterFullscreenFromTouch, { capture: true });
  document.addEventListener("touchend", enterFullscreenFromTouch, { capture: true });
}

function installFullscreenButton() {
  if (!document.fullscreenEnabled || document.getElementById(BUTTON_ID)) {
    return;
  }

  installStyle();

  const button = document.createElement("button");
  button.id = BUTTON_ID;
  button.type = "button";
  button.addEventListener("click", toggleFullscreen);
  document.addEventListener("fullscreenchange", () => updateButton(button));
  updateButton(button);
  document.body.append(button);
}

installBranding();
installTouchFullscreen();
installFullscreenButton();
