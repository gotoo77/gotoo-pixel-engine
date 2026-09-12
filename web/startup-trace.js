import { safeString } from "./diagnostics-core.js";

const tracedWasmScopes = new WeakSet();
const tracedAnimationScopes = new WeakSet();
const tracedDocuments = new WeakSet();

function wasmResourceDetail(input) {
  const value =
    typeof input === "string"
      ? input
      : input && typeof input.url === "string"
        ? input.url
        : "";

  if (!/\.wasm(?:$|[?#])/i.test(value)) return null;

  try {
    const url = new URL(value, "https://gpe.invalid/");
    const parts = url.pathname.split("/").filter(Boolean);
    return parts.at(-1) ?? value;
  } catch {
    return value;
  }
}

function replaceMethod(target, name, wrapperFactory) {
  if (!target || typeof target[name] !== "function") return false;

  const original = target[name];
  const descriptor = Object.getOwnPropertyDescriptor(target, name);
  const replacement = wrapperFactory(original);

  Object.defineProperty(target, name, {
    configurable: descriptor?.configurable ?? true,
    enumerable: descriptor?.enumerable ?? false,
    writable: descriptor?.writable ?? true,
    value: replacement,
  });
  return true;
}

export function installWasmApiTrace(scope, markEvent, traced = tracedWasmScopes) {
  if (!scope || typeof markEvent !== "function" || traced.has(scope)) return;
  traced.add(scope);

  const installed = [];

  try {
    if (
      replaceMethod(scope, "fetch", (original) => async function tracedFetch(...args) {
        const resource = wasmResourceDetail(args[0]);
        if (!resource) return Reflect.apply(original, this, args);

        markEvent("WASM fetch started", resource);
        try {
          const response = await Reflect.apply(original, this, args);
          markEvent(
            "WASM fetch resolved",
            `${resource}; status=${safeString(response?.status, "unknown")}`,
          );
          return response;
        } catch (error) {
          markEvent("WASM fetch rejected", `${resource}; ${safeString(error)}`);
          throw error;
        }
      })
    ) {
      installed.push("fetch");
    }
  } catch (error) {
    markEvent("WASM fetch trace unavailable", safeString(error));
  }

  const wasm = scope.WebAssembly;
  for (const [name, label] of [
    ["instantiateStreaming", "WASM instantiateStreaming"],
    ["instantiate", "WASM instantiate"],
  ]) {
    try {
      if (
        replaceMethod(wasm, name, (original) => async function tracedInstantiate(...args) {
          markEvent(`${label} started`);
          try {
            const result = await Reflect.apply(original, this, args);
            markEvent(`${label} resolved`);
            return result;
          } catch (error) {
            markEvent(`${label} rejected`, safeString(error));
            throw error;
          }
        })
      ) {
        installed.push(name);
      }
    } catch (error) {
      markEvent(`${label} trace unavailable`, safeString(error));
    }
  }

  if (installed.length) {
    markEvent("WASM startup API trace installed", installed.join("/"));
  } else {
    markEvent("WASM startup API trace unavailable", "fetch/WebAssembly APIs missing");
  }
}

export function installFirstAnimationFrameTrace(
  scope,
  markEvent,
  traced = tracedAnimationScopes,
) {
  if (!scope || typeof markEvent !== "function" || traced.has(scope)) return;
  traced.add(scope);

  try {
    let scheduledObserved = false;
    let callbackObserved = false;
    const installed = replaceMethod(
      scope,
      "requestAnimationFrame",
      (original) => function tracedRequestAnimationFrame(callback) {
        if (!scheduledObserved) {
          scheduledObserved = true;
          markEvent("first requestAnimationFrame scheduled");
        }

        if (typeof callback !== "function") {
          return Reflect.apply(original, this, [callback]);
        }

        const wrappedCallback = (timestamp) => {
          if (!callbackObserved) {
            callbackObserved = true;
            markEvent("first requestAnimationFrame callback");
          }
          return callback(timestamp);
        };
        return Reflect.apply(original, this, [wrappedCallback]);
      },
    );

    markEvent(
      installed ? "animation frame trace installed" : "animation frame trace unavailable",
      installed ? null : "requestAnimationFrame missing",
    );
  } catch (error) {
    markEvent("animation frame trace unavailable", safeString(error));
  }
}

function nodeContainsCanvas(node) {
  if (!node) return false;
  if (String(node.tagName ?? "").toLowerCase() === "canvas") return true;
  return typeof node.querySelector === "function" && Boolean(node.querySelector("canvas"));
}

export function installCanvasAttachmentTrace(
  documentObject,
  MutationObserverClass,
  markEvent,
  traced = tracedDocuments,
) {
  if (!documentObject || typeof markEvent !== "function" || traced.has(documentObject)) return;
  traced.add(documentObject);

  if (documentObject.querySelector?.("canvas")) {
    markEvent("canvas already attached to DOM");
    return;
  }

  if (!documentObject.documentElement || typeof MutationObserverClass !== "function") {
    markEvent("canvas DOM trace unavailable", "MutationObserver/documentElement missing");
    return;
  }

  try {
    const observer = new MutationObserverClass((records) => {
      for (const record of records) {
        for (const node of record.addedNodes ?? []) {
          if (!nodeContainsCanvas(node)) continue;
          markEvent("canvas attached to DOM");
          observer.disconnect();
          return;
        }
      }
    });
    observer.observe(documentObject.documentElement, { childList: true, subtree: true });
    markEvent("canvas DOM trace installed");
    return observer;
  } catch (error) {
    markEvent("canvas DOM trace unavailable", safeString(error));
  }
}

export function installWasmWinitStartupTrace(scope, documentObject, markEvent) {
  if (typeof markEvent !== "function") return;

  installWasmApiTrace(scope, markEvent);
  installFirstAnimationFrameTrace(scope, markEvent);
  installCanvasAttachmentTrace(documentObject, scope?.MutationObserver, markEvent);
}
