<template>
  <svg
    width="17"
    height="21"
    viewBox="0 0 17 21"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g clip-path="url(#clip0_318_700)">
      <path
        d="M1.5 1.5L15.5 10.5L1.5 19.5V1.5Z"
        :fill="primaryColor"
        :stroke="primaryColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </g>
    <defs>
      <clipPath id="clip0_318_700">
        <rect
          width="16"
          height="20"
          :fill="primaryColor"
          transform="translate(0.5 0.5)"
        />
      </clipPath>
    </defs>
  </svg>
</template>

<script setup lang="ts">
const primaryColor = "#FFFFFF";
</script>
