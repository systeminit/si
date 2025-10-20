<template>
  <Modal ref="modalRef" :title="title" :size="props.size">
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
        <NewButton icon="x" @click="close"> Cancel </NewButton>
        <NewButton
          class="flex-grow"
          :icon="confirmIcon"
          :tone="confirmTone"
          :disabled="!irreversibleConfirmed"
          :loading="loading"
          :loadingText="loadingText"
          :loadingIcon="loadingIcon"
          :requestStatus="requestStatus"
          @click="emit('confirm')"
        >
          {{ confirmLabel }}
        </NewButton>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import {
  IconNames,
  Modal,
  NewButton,
  ButtonTones,
  useModal,
  VormInput,
} from "@si/vue-lib/design-system";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { nextTick, PropType, ref } from "vue";

const props = defineProps({
  title: { type: String },
  irreversible: { type: Boolean },
  confirmLabel: { type: String, default: "Confirm" },
  confirmIcon: { type: String as PropType<IconNames> },
  confirmTone: { type: String as PropType<ButtonTones> },
  size: {
    type: String as PropType<
      "sm" | "md" | "lg" | "xl" | "2xl" | "4xl" | "4wxl" | "6xl" | "7xl" | "max"
    >,
    default: "lg",
  },
  loading: { type: Boolean },
  loadingText: { type: String },
  loadingIcon: { type: String as PropType<IconNames>, default: "loader" },
  requestStatus: {
    type: [Boolean, Object] as PropType<false | ApiRequestStatus>, // can be false if passing 'someCondition && status'
  },
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
