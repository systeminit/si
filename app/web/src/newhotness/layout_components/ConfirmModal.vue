<template>
  <Modal ref="modalRef" :title="title">
    <div class="max-h-[70vh] overflow-hidden flex flex-col gap-sm">
      <div>
        <slot />
      </div>

      <div class="flex gap-sm">
        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
        <VButton
          class="flex-grow"
          icon="trash"
          tone="destructive"
          @click="emit('confirm')"
        >
          Confirm
        </VButton>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, useModal, VButton } from "@si/vue-lib/design-system";
import { ref } from "vue";

defineProps({
  title: { type: String },
});

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const open = () => {
  // TODO(WENDY) - this is where we reset the confirm thingy
  openModal();
};

const emit = defineEmits<{
  (e: "confirm"): void;
}>();

defineExpose({ open, close });
</script>
