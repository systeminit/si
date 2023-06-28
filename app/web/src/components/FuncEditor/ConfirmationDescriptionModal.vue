<template>
  <Modal
    ref="descriptionsModalRef"
    title="Edit Confirmation Descriptions"
    type="save"
    saveLabel="Save descriptions"
    @save="saveDescriptions"
  >
    <Stack spacing="sm">
      <VormInput
        v-if="schemaVariants.length > 0"
        v-model="selectedVariant"
        label="Asset Type"
        type="dropdown"
        class="flex-1"
        :options="schemaVariants"
      />
      <div v-else>No schema variants configured for this confirmation.</div>

      <Stack v-if="editingFuncDescriptions[selectedVariant]" spacing="sm">
        <VormInput
          v-model="editingName"
          label="Name"
          required
          placeholder="The name of this confirmation..."
        />
        <VormInput
          v-model="editingProvider"
          label="Provider"
          required
          placeholder="The cloud provider used by this confirmation..."
        />
        <VormInput
          v-model="editingSuccessDescription"
          label="Success Description"
          type="textarea"
          placeholder="The message to display when this confirmation succeeds..."
        />
        <VormInput
          v-model="editingFailureDescription"
          label="Failure Description"
          type="textarea"
          placeholder="The message to display when this confirmation fails..."
        />
      </Stack>
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import { computed, toRef, ref, watch } from "vue";
import { VormInput, Stack, Modal, useModal } from "@si/vue-lib/design-system";
import { FuncDescriptionView } from "@/store/func/types";
import { nilId } from "@/utils/nilId";

const props = defineProps<{
  schemaVariants: { label: string; value: string | number | object }[];
  modelValue: FuncDescriptionView[];
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: FuncDescriptionView[]): void;
  (e: "change", v: FuncDescriptionView[]): void;
}>();

const descriptionsModalRef = ref<InstanceType<typeof Modal>>();

const { open: openModal, close } = useModal(descriptionsModalRef);

const open = () => {
  openModal();
};

defineExpose({ open, close });

const schemaVariants = toRef(props, "schemaVariants", []);
const modelValue = toRef(props, "modelValue", []);
const selectedVariant = ref<string>(
  typeof schemaVariants.value?.[0]?.value === "string"
    ? schemaVariants.value?.[0]?.value ?? ""
    : "",
);

const funcDescriptions = computed(() => {
  const descriptionsBySvId: { [key: string]: FuncDescriptionView } = {};

  for (const sv of schemaVariants.value) {
    const { label, value } = sv;
    if (typeof value !== "string") {
      continue;
    }

    const desc = modelValue.value.find((descView) => {
      return descView.schemaVariantId === value;
    }) ?? {
      id: nilId(),
      schemaVariantId: value,
      contents: {
        Confirmation: {
          name: label,
          success_description: "",
          failure_description: "",
          provider: "",
        },
      },
    };

    descriptionsBySvId[value] = desc;
  }

  return descriptionsBySvId;
});

const editingName = ref<string>("");
const editingProvider = ref<string>("");
const editingSuccessDescription = ref<string>("");
const editingFailureDescription = ref<string>("");

const editingFuncDescriptions = ref(funcDescriptions);

const saveCurrentDescription = (svId: string) => {
  const description = editingFuncDescriptions.value[svId];
  if (description) {
    description.contents.Confirmation = {
      name: editingName.value,
      provider: editingProvider.value ?? "",
      success_description: editingSuccessDescription.value ?? "",
      failure_description: editingFailureDescription.value ?? "",
    };

    editingFuncDescriptions.value[svId] = description;
  }
};

watch(
  selectedVariant,
  (sv, prevSv) => {
    if (typeof sv !== "string") {
      return;
    }

    if (typeof prevSv === "string") {
      saveCurrentDescription(prevSv);
    }

    const selectedDescription = editingFuncDescriptions.value[sv];
    if (!selectedDescription) {
      return;
    }

    editingName.value = selectedDescription.contents.Confirmation.name;
    editingProvider.value =
      selectedDescription.contents.Confirmation.provider ?? "";
    editingSuccessDescription.value =
      selectedDescription.contents.Confirmation.success_description ?? "";
    editingFailureDescription.value =
      selectedDescription.contents.Confirmation.failure_description ?? "";
  },
  { immediate: true },
);

const saveDescriptions = () => {
  saveCurrentDescription(selectedVariant.value);
  const newDescriptions = Object.values(editingFuncDescriptions.value);
  emit("update:modelValue", newDescriptions);
  emit("change", newDescriptions);
  close();
};
</script>
