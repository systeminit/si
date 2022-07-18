<template>
  <svg
    width="27"
    height="23"
    viewBox="0 0 27 23"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect y="17" width="8" height="6" rx="1" :fill="primaryColor" />
    <rect x="10" y="17" width="8" height="6" rx="1" :fill="primaryColor" />
    <rect x="7" width="14" height="9" rx="1" :fill="primaryColor" />
    <rect x="19" y="17" width="8" height="6" rx="1" :fill="primaryColor" />
    <rect x="13" y="10" width="2" height="5.25" rx="0.5" :fill="primaryColor" />
    <path
      d="M4 15V14.5C4 13.3954 4.89543 12.5 6 12.5H21C22.1046 12.5 23 13.3954 23 14.5V15"
      :stroke="primaryColor"
      stroke-width="2"
      stroke-miterlimit="3.86874"
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup lang="ts">
const primaryColor = "#FFFFFF";
</script>
