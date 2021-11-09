<template>
  <div>
    <div
      :data-testid="testId"
      class="bg-red-500 text-white flex"
      v-if="message"
    >
      <span class="flex-grow">Error: {{ message }}</span>
      <button @click="clear" class="flex-grow-0">
        <VueFeather type="xicon" />
      </button>
    </div>
    <div
      :data-testid="testId + '-okay'"
      class="invisible w-0 h-0"
      v-if="success"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, } from "vue";
import VueFeather from "vue-feather";

defineProps({
  message: { type: String, default: "" },
  success: { type: Boolean, default: false },
  testId: { type: String, default: "error-message" },
});
const emit = defineEmits(["clear"]);

const clear = () => {
  emit("clear");
};
</script>
