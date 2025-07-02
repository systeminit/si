<template>
  <Modal ref="modalRef" :title="title" size="lg">
    <div class="max-h-[70vh] overflow-hidden flex flex-col gap-sm">
      <slot />

      <VormInput
        v-if="irreversible"
        ref="irreversibleConfirmRef"
        v-model="irreversibleConfirmed"
        noLabel
        type="checkbox"
        class="px-xs"
      >
        I am aware that this action is irreversible and want to proceed.
      </VormInput>

      <div class="flex flex-row items-center gap-sm">
        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
        <VButton
          class="flex-grow"
          icon="trash"
          tone="destructive"
          :disabled="!irreversibleConfirmed"
          @click="emit('confirm')"
        >
          Confirm
        </VButton>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, useModal, VButton, VormInput } from "@si/vue-lib/design-system";
import { nextTick, ref } from "vue";

const props = defineProps({
  title: { type: String },
  irreversible: { type: Boolean },
});

const irreversibleConfirmed = ref(false);
const irreversibleConfirmRef = ref<InstanceType<typeof VormInput>>();

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const open = () => {
  irreversibleConfirmed.value = !props.irreversible;
  openModal();
  // prevent the automatic selection of the checkbox
  nextTick(() => irreversibleConfirmRef.value?.blur());
};

const emit = defineEmits<{
  (e: "confirm"): void;
}>();

defineExpose({ open, close });
</script>
