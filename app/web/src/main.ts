import { Buffer } from "buffer";
import { createApp } from "vue";
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { createHead } from "@vueuse/head";
import VueSafeTeleport from "vue-safe-teleport";
import Toast from "vue-toastification";
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

app.use(Toast); // see https://vue-toastification.maronato.dev/ for some optoins we can set

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.use(VueSafeTeleport);

app.mount("#app");
