<template>
  <div id="app" class="bg-white">
    <router-view class=""></router-view>
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
