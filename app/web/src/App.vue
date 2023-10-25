<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <div class="font-sans">
    <template v-if="route.name === 'auth-connect'">
      <RouterView />
    </template>
    <template
      v-else-if="
        !authStore.userIsLoggedInAndInitialized &&
        (!restoreAuthReqStatus.isRequested ||
          restoreAuthReqStatus.isPending ||
          reconnectAuthReqStatus.isPending)
      "
    >
      <p>restoring auth...</p>
    </template>
    <template v-else>
      <CachedAppNotification />
      <RealtimeConnectionStatus />
      <RouterView :key="selectedWorkspace?.pk" />
      <Teleport to="body">
        <canvas
          id="confetti"
          class="fixed w-full h-full top-0 left-0 pointer-events-none z-100"
        ></canvas>
      </Teleport>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
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
import RealtimeConnectionStatus from "./components/RealtimeConnectionStatus.vue";
import CachedAppNotification from "./components/CachedAppNotification.vue";

useCustomFontsLoadedProvider();

// provides the root theme value to all children, and returns that root theme to use below
const { theme: rootTheme } = useThemeContainer();

useHead(
  computed(() => ({
    bodyAttrs: {
      // add some base classes we need these type classes set for capsize plugin to work throughout
      // and add dark mode style/class
      class: tw`font-sans text-base leading-none ${rootTheme.value}`,
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
  authStore.localLogout(false);
} else {
  authStore.initFromStorage().then();
}

const restoreAuthReqStatus = authStore.getRequestStatus("RESTORE_AUTH");
const reconnectAuthReqStatus = authStore.getRequestStatus("AUTH_RECONNECT");

const workspacesStore = useWorkspacesStore();
const selectedWorkspace = computed(() => workspacesStore.selectedWorkspace);

// initialize the realtime store - which will watch for auth and open/close websocket
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const realtimeStore = useRealtimeStore();
</script>

<style>
.v-popper__arrow-container {
  display: none;
}

.v-popper__inner {
  border-radius: 0px !important;
  border-color: #5a5a5a !important;
  max-width: 80vw;
  overflow-wrap: break-word;
}
</style>
