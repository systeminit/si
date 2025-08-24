// Example filename: tracing.js
import { Buffer } from "buffer";
import { HoneycombWebSDK } from "@honeycombio/opentelemetry-web";
import { DocumentLoadInstrumentation } from "@opentelemetry/instrumentation-document-load";
import { UserInteractionInstrumentation } from "@opentelemetry/instrumentation-user-interaction";
import { LongTaskInstrumentation } from "@opentelemetry/instrumentation-long-task";
import opentelemetry, { Span } from "@opentelemetry/api";
import { mapStackTrace } from "sourcemapped-stacktrace";
import { VueQueryPlugin } from "@tanstack/vue-query";
import VueSecureHTML from "vue-html-secure";

import { ComponentPublicInstance, createApp } from "vue";
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { createHead } from "@vueuse/head";
import VueSafeTeleport from "vue-safe-teleport";
import Toast, { PluginOptions, POSITION } from "vue-toastification";
import "vue-toastification/dist/index.css";
import { FLOATING_VUE_THEMES } from "@si/vue-lib/design-system";

import "@si/vue-lib/tailwind/main.css";
import "@si/vue-lib/tailwind/tailwind.css";

import App from "@/App.vue";
import "./utils/posthog";
import router from "./router";
import store from "./store";

export const APP_MINIMUM_WIDTH = 700;

let otelEndpoint = import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT;
if (!otelEndpoint) otelEndpoint = window.location.host;
const sdk = new HoneycombWebSDK({
  endpoint: `${otelEndpoint}/v1/traces`,
  serviceName: "si-vue",
  skipOptionsValidation: true,
  instrumentations: [
    // we're not auto-instrumenting XMLHttpRequest, we're instrumenting that in pinia_tools.APIRequest
    new DocumentLoadInstrumentation(),
    new UserInteractionInstrumentation({
      shouldPreventSpanCreation: (eventType, element, span) => {
        span.setAttribute("target.tagName", element.tagName);
        span.setAttribute("target.html", element.outerHTML);
      },
    }), // just click events for now
    new LongTaskInstrumentation({
      observerCallback: (span, _longtaskEvent) => {
        span.setAttribute("location.pathname", window.location.pathname);
      },
    }),
  ],
});

sdk.start();

// this is for joi - because we are importing the source rather than the default build made for the browser
globalThis.Buffer = Buffer;

const app = createApp(App);

app.use(createHead());
app.use(router);
app.use(store);
app.use(VueQueryPlugin);
app.use(VueSecureHTML);

const observeError = (message: string, stack: string, components?: string) => {
  const span = opentelemetry.trace.getActiveSpan();

  const _report = (span: Span) => {
    span.setAttribute("error.stacktrace", stack);
    span.setAttribute("error.message", message);
    if (components) span.setAttribute("error.components", components);
  };

  if (span) {
    _report(span);
  } else {
    const tracer = opentelemetry.trace.getTracer("errorHandler");
    tracer.startActiveSpan("error", (span) => {
      _report(span);
      span.end();
    });
  }
};

// This handles local errors
window.onerror = (message, source, lineno, colno, error) => {
  // ignoring these
  if (
    message
      .toString()
      .includes("TypeError: NetworkError when attempting to fetch resource.")
  )
    return;

  if (!error) observeError(message.toString(), "");
  else if (error.stack) observeError(message.toString(), error.stack);
};

// This handles prod-build errors
app.config.errorHandler = (
  err: unknown,
  instance: ComponentPublicInstance | null,
) => {
  // ignoring these
  if (!(err instanceof Error)) return;
  if (
    err.message
      .toString()
      .includes("TypeError: NetworkError when attempting to fetch resource.")
  )
    return;

  if (err.stack) {
    const componentDesc = [];
    if (instance) {
      const components = [instance];
      while (components.length > 0) {
        const inst = components.shift();
        if (!inst) break;
        if (inst.$.type.__name === "AppLayout") break; // dont need anything higher than this

        componentDesc.push(
          `${inst.$.type.__name}: ${JSON.stringify(inst.$props)}`,
        );
        if (inst.$parent) components.push(inst.$parent);
      }
    }
    const componentPath = componentDesc.reverse().join(" > ");

    mapStackTrace(
      err.stack,
      (mappedStack) => {
        const stack = mappedStack.join("\n");
        observeError(err.message.toString(), stack, componentPath);
        if (import.meta.env.VITE_SI_ENV === "local") {
          // eslint-disable-next-line no-console
          console.error(err);
        }
      },
      { cacheGlobally: true },
    );
  }
};

// set the default tooltip delay to show and hide faster
FloatingVue.options.themes.tooltip.delay = { show: 10, hide: 100 };

// we attach to the #app-layout div (in AppLayout.vue) to stay within an overflow hidden div and not mess with page scrollbars
app.use(FloatingVue, {
  container: "#app-layout",
  themes: FLOATING_VUE_THEMES,
});

/* function asyncGetContainer(): Promise<HTMLElement> {
  return new Promise((resolve) => {
    const observer = new MutationObserver((mutations, me) => {
      const myContainer = document.getElementById("konva-container");
      if (myContainer) {
        me.disconnect();
        resolve(myContainer);
      }
    });
    observer.observe(document, {
      childList: true,
      subtree: true,
    });
  });
} */

/**
 * If we have a CONFLICT toast, block merge/up-to-date toasts
 * from overwriting it
 */
/* eslint-disable @typescript-eslint/no-explicit-any */
const filterToasts = (toasts: any[]) => {
  for (const t of toasts) {
    if (t.content.component?.__name === "Conflict") {
      return [t];
    }
    if (t.content.component?.__name === "MaintenanceMode") {
      return [t];
    }
    if (t.content.component?.__name === "UnscheduledDowntime") {
      return [t];
    }
  }
  return toasts;
};

const filterBeforeCreate = (toast: any, toasts: any[]): any | false => {
  if (
    ["MaintenanceMode", "RebaseOnBase"].includes(
      toast.content.component?.__name,
    )
  ) {
    // Basically only have one maintenanace toast in the toast queue
    // at once as they time out serially, which is a bit of a pain with
    // longer timeouts
    if (toasts.length >= 1) {
      return false;
    } else {
      return toast;
    }
  }
  return toast;
};

const options: PluginOptions = {
  newestOnTop: true,
  containerClassName: "diagram-toast-container",
  position: POSITION.TOP_CENTER, // we overriding to push this down
  transition: "si-toast-fade", // works better with overriden position
  icon: false,
  closeButton: false,
  draggable: false,
  hideProgressBar: true,
  timeout: 1500,
  filterToasts,
  filterBeforeCreate,
  // container: asyncGetContainer // right now we cannot make the container a div within nested components that get destroyed on route transitions
  // if we could use that div, we get get TOP_RIGHT position cleanly...
};
app.use(Toast, options); // see https://vue-toastification.maronato.dev/ for some optoins we can set

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.use(VueSafeTeleport);

app.mount("#app");
