<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-full select-none"
  >
    <Navbar />

    <router-view class="overflow-y-hidden" />

    <StatusBar class="flex-initial" />
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisims/Navbar.vue";
import StatusBar from "@/organisims/StatusBar.vue";
</script>
