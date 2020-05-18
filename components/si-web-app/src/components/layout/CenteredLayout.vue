<template>
  <div
    id="centered-layout"
    class="flex justify-center items-center w-full h-full"
  >
    <slot />
  </div>
</template>

<script>
export default {
  name: "CenteredLayout",
};
</script>
