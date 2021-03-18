<template>
  <div id="app">
    <ShortcutsEventBroker>
      <router-view />
    </ShortcutsEventBroker>
  </div>
</template>

<script lang="ts">
import ShortcutsEventBroker from "@/organisims/ShortcutsEventBroker.vue";

export default {
  name: "App",
  components: {
    ShortcutsEventBroker,
  },
};
</script>

<style>
#app {
  font-family: "Roboto", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
