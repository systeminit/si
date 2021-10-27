<template>
  <div class="flex flex-col items-center justify-center h-screen auth-canvas">
    <router-view />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "AuthenticatePage",
});
</script>

<style scoped>
.auth-canvas {
  background-color: #171717;
}
</style>
