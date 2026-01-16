<template>
  <Modal ref="modal" :title="props.title" size="sm">
    <VormInput
      ref="assetNameVorm"
      v-model="assetName"
      :disabled="loading"
      label="Asset Name"
      noLabel
      required
      type="text"
      @enterPressed="submit"
    />
    <VButton :loading="loading" class="mt-sm" @click="submit">{{ props.buttonLabel }} </VButton>
  </Modal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { Modal, VormInput, VButton } from "@si/vue-lib/design-system";

const props = defineProps<{
  title: string;
  buttonLabel: string;
  loading?: boolean;
}>();

const modal = ref<InstanceType<typeof Modal>>();
const assetName = ref("");
const assetNameVorm = ref<InstanceType<typeof VormInput>>();

const submit = () => {
  if (!assetNameVorm.value?.validationState.isError) {
    emit("submit", assetName.value);
  }
};

const reset = () => {
  assetName.value = "";
  assetNameVorm.value?.validationMethods.reset();
};

const setError = (msg: string) => {
  assetNameVorm.value?.setError(msg);
};

defineExpose({
  modal,
  setError,
  reset,
});
const emit = defineEmits(["submit"]);
</script>
