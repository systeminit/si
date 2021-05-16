<template>
  <div id="app">
    <SessionBroker />
    <ShortcutsEventBroker>
      <router-view />
    </ShortcutsEventBroker>
  </div>
</template>

<script lang="ts">
import ShortcutsEventBroker from "@/organisims/ShortcutsEventBroker.vue";
import SessionBroker from "@/organisims/SessionBroker.vue";

export default {
  name: "App",
  components: {
    ShortcutsEventBroker,
    SessionBroker,
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
