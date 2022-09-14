<template>
  <AppLayout>
    <Navbar />

    <div class="w-full h-full flex flex-row relative overflow-hidden">
      <router-view />
    </div>

    <StatusBar class="flex-none" />
    <SiToast />
  </AppLayout>
</template>

<script lang="ts" setup>
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import SiToast from "@/atoms/SiToast.vue";
import AppLayout from "./AppLayout.vue";
</script>
