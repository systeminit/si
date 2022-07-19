<template>
  <svg
    width="14"
    height="14"
    viewBox="0 0 16 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="inline"
    :class="classes"
  >
    <path
      d="M11.4 12.85C12.4769 12.85 13.35 11.9769 13.35 10.9C13.35 9.823 12.4769 8.94995 11.4 8.94995C10.323 8.94995 9.44995 9.823 9.44995 10.9C9.44995 11.9769 10.323 12.85 11.4 12.85Z"
      :stroke="color"
      stroke-width="1"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M3.60002 5.05002C4.67698 5.05002 5.55002 4.17698 5.55002 3.10002C5.55002 2.02307 4.67698 1.15002 3.60002 1.15002C2.52307 1.15002 1.65002 2.02307 1.65002 3.10002C1.65002 4.17698 2.52307 5.05002 3.60002 5.05002Z"
      :stroke="color"
      stroke-width="1"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M3.59998 12.85V5.05005C3.59998 8.28055 6.21948 10.9 9.44998 10.9"
      :stroke="color"
      stroke-width="1"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  // Provided as a hex value.
  color?: string;
  class?: string;
}>();

const color = computed(() => props.color ?? "#FFFFFF");
const classes = computed(() => props.class ?? "");
</script>
