<template>
  <Modal :open="open" size="md" @close="emit('close')">
    <template #title>Controls For Diagram Editor</template>
    <template #content>
      <div class="p-4 flex flex-row place-content-center">
        <ul class="space-y-2">
          <li>Mouse - Add, Create, and Select Components</li>
          <li>Arrow Keys - Nudge Selected Component</li>
          <li>Spacebar - Hold To Drag Scroll</li>
          <li>Esc - Deselect Component</li>
          <li>Backspace/Delete - Delete Selected Component</li>
        </ul>
      </div>
    </template>
  </Modal>
</template>

<script lang="ts" setup>
import Modal from "@/ui-lib/Modal.vue";

defineProps({
  open: { type: Boolean, default: false },
});

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>
