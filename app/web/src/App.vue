<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <Suspense>
    <template #default>
      <div class="font-sans">
        <template v-if="route.name === 'auth-connect'">
          <RouterView />
        </template>
        <template
          v-else-if="
            !authStore.userIsLoggedInAndInitialized &&
            (!restoreAuthReqStatus.isRequested || restoreAuthReqStatus.isPending || reconnectAuthReqStatus.isPending)
          "
        >
          <p>restoring auth...</p>
        </template>
        <template v-else>
          <CachedAppNotification />
          <RouterView :key="selectedWorkspace?.pk" />
        </template>
      </div>
    </template>
    <template #fallback>Loading...</template>
  </Suspense>
  <div v-if="suspenseError">Suspense Error {{ suspenseError }}</div>
</template>

<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref, onErrorCaptured, provide, watch } from "vue";
import "floating-vue/dist/style.css";

import { tw } from "@si/vue-lib";
import { useThemeContainer } from "@si/vue-lib/design-system";
import SiLogoUrlLight from "@si/vue-lib/brand-assets/si-logo-symbol-white-bg.svg?url";
import SiLogoUrlDark from "@si/vue-lib/brand-assets/si-logo-symbol-black-bg.svg?url";
import { useHead } from "@vueuse/head";
import { useRoute } from "vue-router";
import { useCustomFontsLoadedProvider } from "./utils/useFontLoaded";
import { useAuthStore } from "./store/auth.store";
import { useWorkspacesStore } from "./store/workspaces.store";
import { useRealtimeStore } from "./store/realtime/realtime.store";
import CachedAppNotification from "./components/CachedAppNotification.vue";
import { APP_MINIMUM_WIDTH } from "./main";

const suspenseError = ref();
onErrorCaptured((err) => {
  suspenseError.value = err;
});

// this TS magic means that when you call Object.entries
// the "key" will retain its type and will not just be defaulted to "string"
type Entries<T> = {
  [K in keyof T]: [K, T[K]];
}[keyof T][];

declare global {
  interface ObjectConstructor {
    entries<T extends object>(o: T): Entries<T>;
  }
}

useCustomFontsLoadedProvider();

// provides the root theme value to all children, and returns that root theme to use below
const { theme: rootTheme } = useThemeContainer();

// watch the window size to enforce minimum window width
const windowWidth = ref(window.innerWidth);
const windowSizeClasses = computed(() =>
  windowWidth.value < APP_MINIMUM_WIDTH
    ? tw`min-w-[700px] overflow-x-auto` // APP_MINIMUM_WIDTH
    : "",
);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});

const loadingGuard = ref(false);
provide("LOADINGGUARD", {
  loadingGuard,
});

useHead(
  computed(() => ({
    bodyAttrs: {
      // add some base classes we need these type classes set for capsize plugin to work throughout
      // and add dark mode style/class
      class: tw`font-sans text-base leading-none fixed ${rootTheme.value} ${windowSizeClasses.value}`,
    },
    htmlAttrs: {
      style: `color-scheme: ${rootTheme.value};`,
    },
    link: [
      {
        rel: "icon",
        href: rootTheme.value === "light" ? SiLogoUrlLight : SiLogoUrlDark,
      },
    ],
    // set up title template and a default
    titleTemplate: "SI | %s",
    title: "Workspace",
  })),
);

// check for auth token in local storage and initialize auth in store if found
// this token will be automatically injected into API requests
const authStore = useAuthStore();
const route = useRoute();

if (route.name === "auth-connect") {
  // we are just clearing any local login state since we are in the process of logging in again
  authStore.localLogout(false);
} else {
  authStore.initFromStorage();
}

const restoreAuthReqStatus = authStore.getRequestStatus("RESTORE_AUTH");
const reconnectAuthReqStatus = authStore.getRequestStatus("AUTH_RECONNECT");

const workspacesStore = useWorkspacesStore();
const selectedWorkspace = computed(() => workspacesStore.selectedWorkspace);

// initialize the realtime store - which will watch for auth and open/close websocket
const realtimeStore = useRealtimeStore();

const user = computed(() => authStore.user);
const subscribed = ref(false);

watch([selectedWorkspace, user], () => {
  const workspaceId = selectedWorkspace.value?.id;
  const userPk = user.value?.pk;
  if (subscribed.value || workspaceId === undefined || userPk === undefined) return;

  subscribed.value = true;
  // the authStore does not run the activated hook so we let the App handle its subscriptions.
  // This makes sure we always subscribe on a workspace id, but only once, since the id may not be defined at the start but will never change.
  realtimeStore.subscribe("auth", `workspace/${workspaceId}`, [
    {
      eventType: "UserWorkspaceFlagsUpdated",
      callback: (payload) => {
        if (userPk !== payload.userPk) return;
        authStore.updateFlags(payload.flags);
      },
    },
  ]);
});
</script>

<style lang="less">
.v-popper__arrow-container {
  display: none;
}

.v-popper__inner {
  border-radius: 0px !important;
  border-color: #5a5a5a !important;
  max-width: 80vw;
  overflow-wrap: break-word;
}

.v-popper--theme-w-380 > .v-popper__wrapper > .v-popper__inner {
  max-width: 380px;
}

.v-popper--theme-apply-button > .v-popper__wrapper > .v-popper__inner {
  max-width: 390px;
  font-size: 13px;
  line-height: 1.25;
  background-color: #262626 !important;
  border: 1px solid #5a5a5a !important;
  border-radius: 0.25rem !important;
}

.v-popper--theme-attribute-docs > .v-popper__wrapper > .v-popper__inner {
  border-radius: 0.5rem !important;
  max-width: 420px;
}

.v-popper--theme-user-info,
.v-popper--theme-html {
  margin-top: 12px;
  font-style: italic;
  border-radius: 1rem;
}

.v-popper--theme-user-info {
  font-size: 1.25rem;
}

.v-popper--theme-user-info > .v-popper__wrapper > .v-popper__inner {
  border-radius: 0.5rem !important;
  padding-left: 0;
  padding-right: 0;
}

.v-popper--theme-html > .v-popper__wrapper > .v-popper__inner,
.v-popper--theme-attribute-source-icon > .v-popper__wrapper > .v-popper__inner {
  border-radius: 0.5rem !important;
}

.bg-caution-lines-light {
  background: repeating-linear-gradient(-45deg, #fff, #fff 10px, #ccc 10px, #ccc 20px);
}

.bg-caution-lines-dark {
  background: repeating-linear-gradient(-45deg, #000, #000 10px, #333 10px, #333 20px);
}

@keyframes siToastFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@keyframes siToastFadeOut {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
.si-toast-fade-enter-active {
  animation-name: siToastFadeIn;
  animation-duration: 250ms;
  animation-fill-mode: both;
}
.si-toast-fade-leave-active {
  animation-name: siToastFadeOut;
  animation-duration: 250ms;
  animation-fill-mode: both;
}
.si-toast-fade-move {
  transition-timing-function: ease-in-out;
  transition-property: all;
  transition-duration: 400ms;
}
.si-toast-no-defaults {
  padding: 0;
  margin: 0;
  min-width: 0;
  min-height: 0;
}

body.light {
  --toast-text-color: @colors-black;
  --toast-bg-color: @colors-white;

  .v-popper--theme-attribute-docs > .v-popper__wrapper > .v-popper__inner {
    color: #000;
    background-color: #fff;
    border: 1px solid #000;
  }

  .v-popper--theme-html > .v-popper__wrapper > .v-popper__inner,
  .v-popper--theme-attribute-source-icon > .v-popper__wrapper > .v-popper__inner {
    color: #000;
    background-color: #fff;
    border: 1px solid #000;
  }
}
body.dark {
  --toast-text-color: @colors-white;
  --toast-bg-color: @colors-black;

  .v-popper--theme-attribute-docs > .v-popper__wrapper > .v-popper__inner {
    color: #fff;
    background-color: #000;
    border: 1px solid #fff;
    border-color: #fff !important;
  }

  .v-popper--theme-html > .v-popper__wrapper > .v-popper__inner,
  .v-popper--theme-attribute-source-icon > .v-popper__wrapper > .v-popper__inner {
    color: #fff;
    background-color: rgb(0, 0, 0);
    border: 1px solid #fff;
    border-color: #fff !important;
  }
}

.Vue-Toastification__container {
  & .Vue-Toastification__toast {
    background-color: var(--toast-bg-color);
    color: var(--toast-text-color);

    .Vue-Toastification__progress-bar {
      background-color: var(--toast-text-color);
    }
  }
}

.Vue-Toastification__container.diagram-toast-container.top-left,
.Vue-Toastification__container.diagram-toast-container.top-right,
.Vue-Toastification__container.diagram-toast-container.top-center {
  position: absolute; // default is fixed, but we want it positioned within its container, so go absolute
  padding: 0;
  top: 70px; // this puts the toasts 10px below the NavBar at the top of the screen
}

.Vue-Toastification__container.diagram-toast-container.bottom-left,
.Vue-Toastification__container.diagram-toast-container.bottom-right,
.Vue-Toastification__container.diagram-toast-container.bottom-center {
  position: absolute; // default is fixed, but we want it positioned within its container, so go absolute
  padding: 0;
  bottom: 58px; // this puts the toasts 10px above the bottom bar of the Explore page
}

/*
 * Mask text in secret textareas
 *
 * Primary method: -webkit-text-security (works in Chrome, Safari, Edge)
 * Fallback: text-security font (works in Firefox and other browsers)
 *
 * The -webkit-text-security property is nonstandard but widely supported in WebKit/Blink browsers.
 * For browsers that don't support it (mainly Firefox), we use a custom font that renders all
 * characters as bullets.
 *
 * Font source: https://github.com/noppa/text-security
 */
@font-face {
  font-family: "text-security-disc";
  src: url("data:font/woff2;base64,d09GMgABAAAAAAMoAA0AAAAAB2QAAALcAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP0ZGVE0cGh4GYACCahEICjx3CzoAATYCJAMgBCAFhAoHIBuHBhEVnJ3Ifhy4MbWEhv/////nxv+dyv83k8mJCZiYmYnZ/yGBgYGBgYGBwYP///////+h0f8/xv8f/H+H/R+y/z/h/z/+/w/+P+H/H/x/m/0////7/1/+P/j/4P/Z/z/+/7f/f/v/l/8/+v/w/+H/w/8P/z/8f/L/r/9/+v/w/+H/w/8P/z/8f/L/r/9/+//X/7/9/+v/X/4/+f/k/5P/T/4/+f/k/5P/T/4/+f/k/5P/T/4/+f/k/5P/T/4/+f/k/5P/T/4/+f/k/xP/T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP/T/w/8f/E/xP+T/g/8f/E/xP/T/w/8f/E/xAAC")
    format("woff2");
}

.secret-masked-textarea {
  /* Primary: WebKit/Blink browsers (Chrome, Safari, Edge) */
  -webkit-text-security: disc;
  text-security: disc;

  /* Fallback: Font-based masking for Firefox and other browsers */
  /* This font renders all characters as bullet points */
  font-family: "text-security-disc", monospace;
}

/* Override font-family when -webkit-text-security is supported */
@supports (-webkit-text-security: disc) {
  .secret-masked-textarea {
    font-family: inherit;
  }
}
</style>
