<template>
  <Modal ref="modalRef" title="FrieNDA" no-exit>
    <Stack>
      <FrieNDAContent />
      <VButton2 tone="success" icon="thumbs-up" @click="agreeButtonHandler">
        No problem!
      </VButton2>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, Stack, useModal, VButton2 } from "@si/vue-lib/design-system";
import { ref } from "vue";

import { VueComponent as FrieNDAContent } from "@/content/frienda.md";
import { useOnboardingStore } from "@/store/onboarding.store";

const onboardingStore = useOnboardingStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);
defineExpose({ open, close });
function agreeButtonHandler() {
  onboardingStore.acknowledgeFrienda();
  close();
}
</script>
