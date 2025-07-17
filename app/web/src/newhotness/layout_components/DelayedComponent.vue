<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <slot v-if="show" />
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";

const DELAY_MS = 300;

const show = ref(false);
onMounted(() => {
  setTimeout(() => {
    show.value = true;
  }, DELAY_MS);
});
</script>
