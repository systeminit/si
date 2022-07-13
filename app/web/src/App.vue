<template>
  <div id="app" class="h-max w-full">
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
