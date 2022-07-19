<template>
  <svg
    width="13"
    height="13"
    viewBox="0 0 13 13"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="inline"
    :class="classes"
  >
    <path
      d="M2.59998 0.650024V8.45002"
      :stroke="color"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M10.4 4.55002C11.4769 4.55002 12.35 3.67698 12.35 2.60002C12.35 1.52307 11.4769 0.650024 10.4 0.650024C9.323 0.650024 8.44995 1.52307 8.44995 2.60002C8.44995 3.67698 9.323 4.55002 10.4 4.55002Z"
      :stroke="color"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M2.60002 12.35C3.67698 12.35 4.55002 11.4769 4.55002 10.4C4.55002 9.323 3.67698 8.44995 2.60002 8.44995C1.52307 8.44995 0.650024 9.323 0.650024 10.4C0.650024 11.4769 1.52307 12.35 2.60002 12.35Z"
      :stroke="color"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M10.4 4.55005C10.4 7.78055 7.78055 10.4 4.55005 10.4"
      :stroke="color"
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
