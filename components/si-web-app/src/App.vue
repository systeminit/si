<template>
  <div id="app" class="w-screen h-screen bg-white">
    <router-view class="w-full"></router-view>
  </div>
</template>

<script lang="ts">
import { telemetry } from "@/utils/telemetry";
import { auth } from "@/utils/auth";

export default {
  name: "App",
};

window.addEventListener("beforeunload", () => {
  telemetry.end();
});
</script>

<style>
#app {
  font-family: "Roboto", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
