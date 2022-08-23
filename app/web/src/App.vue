<template>
  <!-- <SessionBroker /> -->
  <!-- <ShortcutsEventBroker> -->
  <router-view />
  <!-- </ShortcutsEventBroker> -->
</template>

<script lang="ts" setup>
import { refFrom } from "vuse-rx/src";
import { computed, onBeforeMount } from "vue";
import "floating-vue/dist/style.css";

import { restoreFromSession } from "@/observable/session_state";
import { ThemeService } from "@/service/theme";
import { Theme } from "@/observable/theme";
import { useThemeProvider } from "./composables/injectTheme";
import { useCustomFontsLoadedProvider } from "./composables/useFontLoaded";

onBeforeMount(restoreFromSession);

const theme = refFrom<Theme>(ThemeService.currentTheme());
useThemeProvider(computed(() => theme.value?.value || "light"));

useCustomFontsLoadedProvider();
</script>

<style>
#app {
  font-family: "Inter", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.vue-feather {
  display: flex !important;
}

.v-popper__arrow-container {
  display: none;
}

.v-popper__inner {
  border-radius: 0px !important;
  border-color: #5a5a5a !important;
}
</style>
