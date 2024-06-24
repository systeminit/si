<template>
  <Modal ref="modal" size="sm" :title="props.title">
    <VormInput
      ref="assetNameVorm"
      v-model="assetName"
      type="text"
      label="Asset Name"
      required
      @enterPressed="submit"
    />
    <VButton class="mt-md" @click="submit">{{ props.buttonLabel }}</VButton>
  </Modal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { Modal, VormInput, VButton } from "@si/vue-lib/design-system";

const props = defineProps<{
  title: string;
  buttonLabel: string;
}>();

const modal = ref<InstanceType<typeof Modal>>();
const assetName = ref("");
const assetNameVorm = ref<InstanceType<typeof VormInput>>();

const submit = () => {
  emit("submit", assetName.value);
};

const setError = (msg: string) => {
  if (assetNameVorm.value) {
    assetNameVorm.value.validationState.isError = true;
    assetNameVorm.value.validationState.errorMessage = msg;
  }
};

defineExpose({
  modal,
  setError,
});
const emit = defineEmits(["submit"]);
</script>
