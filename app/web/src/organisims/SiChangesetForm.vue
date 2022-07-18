<template>
  <div class="w-full border-gray-300 border-b-2 p-2">
    <SiTextBox2 title="Change Set" class="w-3/4" />
    <button class="w-1/4">Apply</button>
    <button>Commit</button>
    <button>Discard</button>
  </div>
</template>

<script setup lang="ts">
import SiTextBox2 from "@/atoms/SiTextBox2.vue";
</script>
