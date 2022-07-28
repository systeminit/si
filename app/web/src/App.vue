<template>
  <!-- <SessionBroker /> -->
  <!-- <ShortcutsEventBroker> -->
  <router-view />
  <!-- </ShortcutsEventBroker> -->
</template>

<script lang="ts" setup>
import { restoreFromSession } from "@/observable/session_state";
import "floating-vue/dist/style.css";
import { onBeforeMount } from "vue";
import { useThemeProvider } from "./composables/injectTheme";

onBeforeMount(restoreFromSession);

useThemeProvider();
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
