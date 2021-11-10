<template>
  <div>
    <div v-if="message" :data-test="test" class="bg-red-500 text-white flex">
      <span class="flex-grow">Error: {{ message }}</span>
      <button class="flex-grow-0" @click="clear">
        <VueFeather type="xicon" />
      </button>
    </div>
    <div
      v-if="success"
      :data-testid="testId + '-okay'"
      class="invisible w-0 h-0"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
import VueFeather from "vue-feather";

defineProps({
  message: { type: String, default: "" },
  success: { type: Boolean, default: false },
  test: { type: String, default: "error-message" },
});
const emit = defineEmits(["clear"]);

const clear = () => {
  emit("clear");
};
</script>
