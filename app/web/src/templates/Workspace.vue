<template>
  <div id="workspaces" class="flex flex-col w-full h-full select-none bg-white">
    <Navbar />

    <router-view />
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisims/Navbar.vue";
</script>
