<template>
  <div ref="divRef" v-tooltip="tooltip" class="truncate">
    <slot />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";

const divRef = ref<HTMLElement>();

const tooltip = computed(() => {
  if (divRef.value && divRef.value.clientWidth < divRef.value.scrollWidth) {
    return {
      content: divRef.value.innerText,
    };
  } else return {};
});
</script>
