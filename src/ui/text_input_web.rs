use crate::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WebTextInputSnapshot {
    pub focused: bool,
    pub value: String,
    pub submitted: bool,
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::{Rect, WebTextInputSnapshot};
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(inline_js = r#"
let gpeUiTextInputFrame = 0;
const gpeUiTextInputSubmitted = new Set();

function inputId(surfaceId, ordinal) {
    return "gpe-ui-text-input-" + surfaceId + "-" + ordinal;
}

function canvasViewport(canvas, framebufferWidth, framebufferHeight) {
    const surfaceWidth = canvas.width;
    const surfaceHeight = canvas.height;
    if (!surfaceWidth || !surfaceHeight || !framebufferWidth || !framebufferHeight) {
        return null;
    }

    const rawScale = Math.min(
        surfaceWidth / framebufferWidth,
        surfaceHeight / framebufferHeight
    );
    const integerScale = Math.floor(rawScale);
    const scale =
        rawScale >= 1 && Math.abs(rawScale - integerScale) < Number.EPSILON
            ? integerScale
            : rawScale;
    const width = Math.round(framebufferWidth * scale);
    const height = Math.round(framebufferHeight * scale);
    return {
        x: Math.trunc((surfaceWidth - width) / 2),
        y: Math.trunc((surfaceHeight - height) / 2),
        width,
        height,
        surfaceWidth,
        surfaceHeight,
    };
}

export function gpeUiTextInputBeginFrame() {
    gpeUiTextInputFrame += 1;
}

export function gpeUiTextInputEnsure(
    surfaceId,
    ordinal,
    logicalX,
    logicalY,
    logicalWidth,
    logicalHeight,
    framebufferWidth,
    framebufferHeight,
    value,
    maxChars,
    enterHint,
    ariaLabel
) {
    const canvas = document.querySelector("canvas");
    if (!canvas) {
        return;
    }
    const viewport = canvasViewport(canvas, framebufferWidth, framebufferHeight);
    if (!viewport) {
        return;
    }

    const id = inputId(surfaceId, ordinal);
    let input = document.getElementById(id);
    if (!input) {
        input = document.createElement("input");
        input.id = id;
        input.type = "text";
        input.autocomplete = "off";
        input.spellcheck = false;
        input.setAttribute("autocapitalize", "sentences");
        input.setAttribute("autocorrect", "off");
        input.dataset.gpeUiTextInput = "1";

        input.addEventListener("keydown", (event) => {
            if (event.key === "Enter") {
                event.preventDefault();
                gpeUiTextInputSubmitted.add(id);
                input.blur();
            }
        });

        document.body.appendChild(input);
    }

    input.dataset.gpeUiFrame = String(gpeUiTextInputFrame);
    input.setAttribute("enterkeyhint", enterHint);
    input.setAttribute("aria-label", ariaLabel || "Text input");
    if (maxChars > 0) {
        input.maxLength = maxChars;
    } else {
        input.removeAttribute("maxlength");
    }

    const canvasRect = canvas.getBoundingClientRect();
    const cssScaleX = canvasRect.width / viewport.surfaceWidth;
    const cssScaleY = canvasRect.height / viewport.surfaceHeight;
    const framebufferScaleX = viewport.width / framebufferWidth;
    const framebufferScaleY = viewport.height / framebufferHeight;

    const surfaceX = viewport.x + logicalX * framebufferScaleX;
    const surfaceY = viewport.y + logicalY * framebufferScaleY;
    const surfaceW = logicalWidth * framebufferScaleX;
    const surfaceH = logicalHeight * framebufferScaleY;

    input.style.position = "fixed";
    input.style.left = (canvasRect.left + surfaceX * cssScaleX) + "px";
    input.style.top = (canvasRect.top + surfaceY * cssScaleY) + "px";
    input.style.width = (surfaceW * cssScaleX) + "px";
    input.style.height = (surfaceH * cssScaleY) + "px";
    input.style.zIndex = "2147483646";
    input.style.boxSizing = "border-box";
    input.style.margin = "0";
    input.style.padding = "0";
    input.style.border = "0";
    input.style.outline = "none";
    input.style.background = "transparent";
    input.style.color = "transparent";
    input.style.caretColor = "transparent";
    input.style.opacity = "0.02";
    input.style.fontSize = "16px";
    input.style.lineHeight = "1";
    input.style.webkitAppearance = "none";

    if (document.activeElement !== input && input.value !== value) {
        input.value = value;
    }
}

export function gpeUiTextInputFocused(surfaceId, ordinal) {
    const active = document.activeElement;
    return active ? active.id === inputId(surfaceId, ordinal) : false;
}

export function gpeUiTextInputValue(surfaceId, ordinal) {
    const input = document.getElementById(inputId(surfaceId, ordinal));
    return input ? input.value : "";
}

export function gpeUiTextInputTakeSubmitted(surfaceId, ordinal) {
    const id = inputId(surfaceId, ordinal);
    if (!gpeUiTextInputSubmitted.has(id)) {
        return false;
    }
    gpeUiTextInputSubmitted.delete(id);
    return true;
}

export function gpeUiTextInputEndFrame() {
    document.querySelectorAll("[data-gpe-ui-text-input='1']").forEach((input) => {
        if (input.dataset.gpeUiFrame !== String(gpeUiTextInputFrame)) {
            gpeUiTextInputSubmitted.delete(input.id);
            input.remove();
        }
    });
}
"#)]
    extern "C" {
        #[wasm_bindgen(js_name = gpeUiTextInputBeginFrame)]
        fn js_begin_frame();

        #[wasm_bindgen(js_name = gpeUiTextInputEnsure)]
        fn js_ensure(
            surface_id: u32,
            ordinal: u32,
            logical_x: i32,
            logical_y: i32,
            logical_width: u32,
            logical_height: u32,
            framebuffer_width: u32,
            framebuffer_height: u32,
            value: &str,
            max_chars: u32,
            enter_hint: &str,
            aria_label: &str,
        );

        #[wasm_bindgen(js_name = gpeUiTextInputFocused)]
        fn js_focused(surface_id: u32, ordinal: u32) -> bool;

        #[wasm_bindgen(js_name = gpeUiTextInputValue)]
        fn js_value(surface_id: u32, ordinal: u32) -> String;

        #[wasm_bindgen(js_name = gpeUiTextInputTakeSubmitted)]
        fn js_take_submitted(surface_id: u32, ordinal: u32) -> bool;

        #[wasm_bindgen(js_name = gpeUiTextInputEndFrame)]
        fn js_end_frame();
    }

    pub(crate) fn begin_frame() {
        js_begin_frame();
    }

    pub(crate) fn sync(
        surface_id: u32,
        ordinal: usize,
        rect: Rect,
        framebuffer_width: u32,
        framebuffer_height: u32,
        value: &str,
        max_chars: Option<usize>,
        enter_hint: &str,
        aria_label: &str,
    ) -> WebTextInputSnapshot {
        let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
        let max_chars = max_chars
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        js_ensure(
            surface_id,
            ordinal,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            framebuffer_width,
            framebuffer_height,
            value,
            max_chars,
            enter_hint,
            aria_label,
        );
        WebTextInputSnapshot {
            focused: js_focused(surface_id, ordinal),
            value: js_value(surface_id, ordinal),
            submitted: js_take_submitted(surface_id, ordinal),
        }
    }

    pub(crate) fn end_frame() {
        js_end_frame();
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::{Rect, WebTextInputSnapshot};

    pub(crate) fn begin_frame() {}

    pub(crate) fn sync(
        _surface_id: u32,
        _ordinal: usize,
        _rect: Rect,
        _framebuffer_width: u32,
        _framebuffer_height: u32,
        value: &str,
        _max_chars: Option<usize>,
        _enter_hint: &str,
        _aria_label: &str,
    ) -> WebTextInputSnapshot {
        WebTextInputSnapshot {
            focused: false,
            value: value.to_owned(),
            submitted: false,
        }
    }

    pub(crate) fn end_frame() {}
}

pub(crate) use imp::{begin_frame, end_frame, sync};
