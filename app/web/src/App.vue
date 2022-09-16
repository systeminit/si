<template>
  <router-view />
</template>

<script lang="ts" setup>
import { refFrom } from "vuse-rx/src";
import { computed, onBeforeMount } from "vue";
import "floating-vue/dist/style.css";

import { useHead } from "@vueuse/head";
import { restoreFromSession } from "@/observable/session_state";
import { ThemeService } from "@/service/theme";
import { Theme } from "@/observable/theme";
import { useThemeProvider } from "./composables/injectTheme";
import { useCustomFontsLoadedProvider } from "./composables/useFontLoaded";
import { tw } from "./utils/style_helpers";

onBeforeMount(restoreFromSession);

const theme = refFrom<Theme>(ThemeService.currentTheme());
const themeValue = computed(() => theme.value?.value || "light");
useThemeProvider(themeValue);

useCustomFontsLoadedProvider();

useHead(
  computed(() => ({
    bodyAttrs: {
      // add some base classes we need these type classes set for capsize plugin to work throughout
      // and add dark mode style/class
      class: tw`font-sans text-base leading-none ${themeValue.value}`,
      style: `color-scheme: ${themeValue.value};`,
    },
    // set up title template and a default
    titleTemplate: "%s | System Init",
    title: "DevOps without papercuts",
  })),
);
</script>

<style>
html,
body {
  margin: 0;
  padding: 0;
  height: 100%;
  width: 100%;
  background: transparent;
}

#app {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.v-popper__arrow-container {
  display: none;
}

.v-popper__inner {
  border-radius: 0px !important;
  border-color: #5a5a5a !important;
}
</style>
