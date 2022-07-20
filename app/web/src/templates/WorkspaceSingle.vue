<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-full select-none bg-white dark:bg-#[333333]"
  >
    <Navbar />

    <router-view />

    <div class="pointer-events-auto absolute bottom-0 z-10 w-screen">
      <StatusBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisims/Navbar.vue";
import StatusBar from "@/organisims/StatusBar.vue";
</script>
