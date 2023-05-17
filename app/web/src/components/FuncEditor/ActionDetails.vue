<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Run this action on the selected assets of type below.
    </h1>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Kind of Action:
    </h2>
    <SelectMenu
      v-model="selectedKind"
      class="flex-auto"
      :options="kindOptions"
      :disabled="disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Assets of Type:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="assets of type"
      :options="schemaVariantOptions"
      :disabled="disabled"
      @change="updateAssociations"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, toRef } from "vue";
import { storeToRefs } from "pinia";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { ActionAssociations } from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { ActionKind } from "@/store/fixes.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: ActionAssociations;
  disabled?: boolean;
}>();

const kindToOption = (kind: string): Option => ({
  label: kind,
  value: kind,
});

const generateKindOptions = () => {
  const options: Option[] = [];
  for (const kind of Object.values(ActionKind)) {
    options.push(kindToOption(kind as ActionKind));
  }
  return options;
};

const modelValue = toRef(props, "modelValue");

const kindOptions = generateKindOptions();

const selectedKind = ref<Option>(
  kindToOption(modelValue.value?.kind ?? "other"),
);

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, modelValue.value.schemaVariantIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: ActionAssociations): void;
  (e: "change", v: ActionAssociations): void;
}>();

watch(
  [modelValue, schemaVariantOptions, componentOptions],
  ([mv, svOpts]) => {
    selectedVariants.value = toOptionValues(svOpts, mv.schemaVariantIds);
  },
  { immediate: true },
);

const updateAssociations = () => {
  const associations: ActionAssociations = {
    kind: selectedKind.value.value as ActionKind,
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as string,
    ),
    type: "action",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
