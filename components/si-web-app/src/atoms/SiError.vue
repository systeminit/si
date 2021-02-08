<template>
  <div>
    <div :data-testid="testId" class="bg-red-500 text-white" v-if="message">
      Error: {{ message }}
    </div>
    <div
      :data-testid="testId + '-okay'"
      class="invisible w-0 h-0"
      v-if="success"
    ></div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "SiError",
  props: {
    message: { type: String, default: "" },
    success: { type: Boolean, default: false },
    testId: { type: String, default: "error-message" },
  },
});
</script>
