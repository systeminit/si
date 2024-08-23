// Example filename: tracing.js
import { Buffer } from "buffer";
import { HoneycombWebSDK } from "@honeycombio/opentelemetry-web";
import { DocumentLoadInstrumentation } from "@opentelemetry/instrumentation-document-load";
import { UserInteractionInstrumentation } from "@opentelemetry/instrumentation-user-interaction";
import { LongTaskInstrumentation } from "@opentelemetry/instrumentation-long-task";
import opentelemetry, { Span } from "@opentelemetry/api";

import { createApp } from "vue";
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { createHead } from "@vueuse/head";
import VueSafeTeleport from "vue-safe-teleport";
import Toast, { PluginOptions, POSITION } from "vue-toastification";
import "vue-toastification/dist/index.css";

import "@si/vue-lib/tailwind/main.css";
import "@si/vue-lib/tailwind/tailwind.css";

import App from "@/App.vue";
import { getProjectEnvVariables } from "./shared/dynamicEnvVars";
import "./utils/posthog";
import router from "./router";
import store from "./store";

const { envVariables } = getProjectEnvVariables();

let otelEndpoint =
  envVariables.VITE_OTEL_EXPORTER_OTLP_ENDPOINT ??
  import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT;
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

window.onerror = (message, source, lineno, colno, error) => {
  const span = opentelemetry.trace.getActiveSpan();

  const _report = (span: Span) => {
    span.setAttribute("error.stacktrace", error?.stack || "");
    span.setAttribute("error.message", message.toString());
    span.setAttribute("error.source", source || "");
    span.setAttribute("error.lineno", lineno || "");
    span.setAttribute("error.colno", colno || "");
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

// seemingly this doesnt do anything at all
// app.config.errorHandler = (err, instance, info) => {};

// set the default tooltip delay to show and hide faster
FloatingVue.options.themes.tooltip.delay = { show: 10, hide: 100 };

// we attach to the #app-layout div (in AppLayout.vue) to stay within an overflow hidden div and not mess with page scrollbars
app.use(FloatingVue, {
  container: "#app-layout",
  themes: {
    html: {
      $extend: "tooltip",
      html: true,
    },
    "instant-show": {
      $extend: "tooltip",
      instantMove: true,
      delay: { show: 0, hide: 100 },
    },
    "user-info": {
      $extend: "instant-show",
      html: true,
    },
    "w-380": {
      $extend: "tooltip",
    },
    "attribute-source-icon": {
      $extend: "tooltip",
    },
  },
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
  }
  return toasts;
};

const filterBeforeCreate = (toast: any, toasts: any[]): any | false => {
  if (toast.content.component.__name === "MaintenanceMode") {
    // Basically only have one maintenanace toast in the toast queue
    // at once as they time out serially, which is a bit of a pain with
    // longer timeouts
    if (toasts.length >= 1) {
      return false;
    } else {
      return toast;
    }
  }
};

const options: PluginOptions = {
  newestOnTop: true,
  containerClassName: "diagram-toast-container",
  position: POSITION.TOP_CENTER, // we overriding to push this down, BOTTOM is useless now
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
