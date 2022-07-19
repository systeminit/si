<template>
  <svg
    width="30"
    height="30"
    viewBox="0 0 30 30"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="2" y="20" width="8" height="7" rx="1" :fill="color" />
    <rect x="11" y="20" width="8" height="7" rx="1" :fill="color" />
    <rect x="8" y="3" width="14" height="9" rx="1" :fill="color" />
    <rect x="20" y="20" width="8" height="7" rx="1" :fill="color" />
    <rect x="14" y="13" width="2" height="6" rx="0.5" :fill="color" />
    <path
      d="M6 18V17.5C6 16.3954 6.89543 15.5 8 15.5H22C23.1046 15.5 24 16.3954 24 17.5V18"
      :stroke="color"
      stroke-width="2"
      stroke-miterlimit="3.86874"
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  // Provided as a hex value.
  color?: string;
}>();

const color = computed(() => {
  if (props.color) {
    return props.color;
  }
  return "#FFFFFF";
});
</script>
