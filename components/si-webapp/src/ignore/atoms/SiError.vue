<template>
  <div>
    <div
      :data-testid="testId"
      class="bg-red-500 text-white flex"
      v-if="message"
    >
      <span class="flex-grow">Error: {{ message }}</span>
      <button @click="clear" class="flex-grow-0">
        <XIcon />
      </button>
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
import { XIcon } from "vue-feather-icons";

export default Vue.extend({
  name: "SiError",
  components: {
    XIcon,
  },
  props: {
    message: { type: String, default: "" },
    success: { type: Boolean, default: false },
    testId: { type: String, default: "error-message" },
  },
  methods: {
    clear() {
      this.$emit("clear");
    },
  },
});
</script>
