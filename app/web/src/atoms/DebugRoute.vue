<template>
  <div :data-test="testId" class="invisible w-0 h-0">
    {{ $route.fullPath }}
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "DebugRoute",
  props: {
    testId: {
      type: String,
      default: "location-display",
    },
  },
});
</script>
