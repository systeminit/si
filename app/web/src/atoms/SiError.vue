<template>
  <div>
    <div
      v-if="message"
      :data-test="test"
      class="bg-destructive-500 text-white flex"
    >
      <span class="flex-grow">Error: {{ message }}</span>
      <button class="flex-grow-0" @click="clear">
        <Icon name="x-circle" />
      </button>
    </div>
    <div
      v-if="success"
      :data-testid="test + '-okay'"
      class="invisible w-0 h-0"
    ></div>
  </div>
</template>

<script setup lang="ts">
import Icon from "@/ui-lib/Icon.vue";

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
