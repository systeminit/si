import { Buffer } from "buffer";
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
import "./utils/posthog";
import router from "./router";
import store from "./store";

// this is for joi - because we are importing the source rather than the default build made for the browser
globalThis.Buffer = Buffer;

const app = createApp(App);

app.use(createHead());
app.use(router);
app.use(store);

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
  // container: asyncGetContainer // right now we cannot make the container a div within nested components that get destroyed on route transitions
  // if we could use that div, we get get TOP_RIGHT position cleanly...
};
app.use(Toast, options); // see https://vue-toastification.maronato.dev/ for some optoins we can set

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.use(VueSafeTeleport);

app.mount("#app");
