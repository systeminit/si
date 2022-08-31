<template>
  <AppLayout>
    <Navbar />

    <router-view class="overflow-hidden" />

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
