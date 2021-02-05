<template>
  <div :data-testid="testId" class="invisible w-0 h-0">
    {{ $route.fullPath }}
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "DebugRoute",
  props: {
    testId: {
      type: String,
      default: "location-display",
    },
  },
});
</script>
