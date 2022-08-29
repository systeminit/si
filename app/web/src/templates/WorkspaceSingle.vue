<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-screen select-none"
  >
    <Navbar />

    <router-view class="overflow-hidden" />

    <StatusBar class="flex-none" />
    <SiToast />
  </div>
</template>

<script lang="ts" setup>
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import SiToast from "@/atoms/SiToast.vue";
</script>
