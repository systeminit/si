<template>
  <Modal ref="modalRef" title="Contribute Assets">
    <Stack>
      <div
        v-if="enableContributeButton"
        class="flex flex-col gap-2xs max-h-72 overflow-auto"
      >
        {{ props.contributeRequest.name }}
      </div>
      <ErrorMessage
        v-if="contributeModuleReqStatus.isError"
        :requestStatus="contributeModuleReqStatus"
      />
      <template
        v-if="isPrivateModule && featureFlagsStore.PRIVATE_SCOPED_MODULES"
      >
        <p>
          By clicking the 'Contribute to System Initiative' button, you confirm
          this is a private module. System Initiative will not distribute
          private modules.
        </p>
      </template>
      <template v-else>
        <p>
          Everything you contribute will receive a code review, and we will
          reach out if we have any questions or concerns. Assuming things look
          good, we will then include your asset in a future version of System
          Initiative!
        </p>
        <p>
          By clicking the 'Contribute to System Initiative' button, you agree to
          license any code submitted under the terms of the
          <a
            class="text-action-500"
            href="https://www.apache.org/licenses/LICENSE-2.0"
            >Apache License, Version 2.0</a
          >, and that you intend for System Initiative, Inc. to distribute it.
        </p>
      </template>

      <VormInput
        v-if="featureFlagsStore.PRIVATE_SCOPED_MODULES"
        v-model="isPrivateModule"
        type="checkbox"
        >This is a private module
      </VormInput>
      <VButton
        :disabled="!enableContributeButton"
        :loadingText="_.sample(contributeLoadingTexts)"
        :requestStatus="contributeModuleReqStatus"
        label="Contribute to System Initiative"
        icon="cloud-upload"
        size="sm"
        tone="action"
        @click="contributeAssets"
      />
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import {
  Modal,
  VButton,
  useModal,
  Stack,
  ErrorMessage,
  VormInput,
} from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { useModuleStore } from "@/store/module.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ModuleContributeRequest } from "@/api/sdf/dal/module";

const moduleStore = useModuleStore();
const featureFlagsStore = useFeatureFlagsStore();
const modalRef = ref<InstanceType<typeof Modal>>();
const contributeModuleReqStatus = moduleStore.getRequestStatus("CONTRIBUTE");

const props = defineProps<{ contributeRequest: ModuleContributeRequest }>();

const emits = defineEmits<{
  (e: "contributeSuccess", isPrivate: boolean): void;
}>();

const isPrivateModule = ref(false);

const contributeLoadingTexts = [
  "Engaging Photon Torpedos...",
  "Reticulating Splines...",
  "Revolutionizing DevOps...",
  "Calibrating Hyperspace Matrix...",
  "Syncing Neural Circuitry...",
  "Optimizing Tachyon Weave...",
  "Tuning Fractal Harmonics...",
  "Reshuffling Multiverse Threads...",
  "Harmonizing Subspace Arrays...",
  "Modulating Cybernetic Matrices...",
  "Configuring Exo-Geometric Arrays...",
  "Initializing Flux Capacitors...",
  "Balancing Subatomic Resonance...",
  "Fine-tuning Quantum Entanglement...",
  "Matrixing Hyperdimensional Grids...",
  "Coalescing Esoteric Code...",
  "Syncopating Quantum Flux...",
  "Reformatting Reality Lattice...",
  "Fine-tuning Temporal Flux...",
  "Syncing Cosmic Harmonics...",
];

const { open: openModal, close } = useModal(modalRef);
const open = () => {
  isPrivateModule.value = false;
  openModal();
};

const isOpen = computed(() => modalRef.value?.isOpen);

defineExpose({ open, close, isOpen });

const enableContributeButton = computed(() => {
  return props.contributeRequest;
});

const contributeAssets = async () => {
  const updatedRequest = {
    ...props.contributeRequest,
    isPrivateModule: isPrivateModule.value || false,
  };
  const result = await moduleStore.CONTRIBUTE(updatedRequest);
  if (result.result.success) {
    emits("contributeSuccess", updatedRequest.isPrivateModule);
    close();
    await moduleStore.LOAD_LOCAL_MODULES();
  }
};
</script>
