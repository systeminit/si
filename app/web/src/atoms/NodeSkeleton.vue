<template>
  <svg
    width="75"
    height="50"
    viewBox="0 0 75 50"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M0 3.99952C0 1.79038 1.79086 -0.000488281 4 -0.000488281H71C73.2091 -0.000488281 75 1.79037 75 3.99951V11.7018H0V3.99952Z"
      :class="'fill-current ' + colorClass"
    />
    <path
      d="M0.5 12.2008H74.5V45.9993C74.5 47.9323 72.933 49.4993 71 49.4993H4C2.067 49.4993 0.5 47.9323 0.5 45.9993V12.2008Z"
      :class="'stroke-current ' + colorClass"
    />
    <rect
      opacity="0.75"
      x="15"
      y="4"
      width="45"
      height="5"
      rx="1"
      fill="white"
    />
    <rect
      opacity="0.75"
      x="15"
      y="19"
      width="45"
      height="5"
      rx="1"
      fill="#BDBDBD"
    />
  </svg>
</template>

<script setup lang="ts">
defineProps<{
  colorClass: `text-${string}`; // Using text-* classes here because it seemed to be the easiest way to leverage tailwind colors for this svg
}>();
</script>
