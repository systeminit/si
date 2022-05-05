<template>
  <div id="app">
    <!-- <SessionBroker /> -->
    <!-- <ShortcutsEventBroker> -->
    <router-view />
    <!-- </ShortcutsEventBroker> -->
  </div>
</template>

<script lang="ts">
// import ShortcutsEventBroker from "./organisims/ShortcutsEventBroker.vue";
import { restoreFromSession } from "@/observable/session_state";
import "floating-vue/dist/style.css";

export default {
  name: "App",
  components: {
    // ShortcutsEventBroker,
  },
  beforeMount() {
    restoreFromSession();
  },
};
</script>

<style>
#app {
  font-family: "Roboto", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
.vue-feather {
  display: flex !important;
}
</style>
