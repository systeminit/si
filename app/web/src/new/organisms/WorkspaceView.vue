<template>
  <div class="flex flex-col items-center justify-center h-screen bg-white">
    <div class="text-center">
      <p class="text-5xl text-black">New home!</p>
    </div>

    <div class="flex m-6 border-2 p-2 rounded-lg border-black">
      <OldAppSwitch />
    </div>
  </div>
</template>

<script setup lang="ts">
import OldAppSwitch from "@/new/atoms/OldAppSwitch.vue";
</script>
