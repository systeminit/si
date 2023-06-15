<template>
  <div>
    <div class="p-3 flex flex-col gap-2">
      <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
        Run this validation on the selected schema variant attributes below.
      </h1>

      <h2 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Schema Variant and Attribute:
      </h2>
      <SelectMenu
        v-model="selectedVariant"
        class="flex-auto"
        :options="schemaVariantOptions ?? []"
      />
      <SelectMenu
        v-model="selectedProp"
        class="flex-auto"
        :options="propOptions"
      />
      <VButton
        icon="plus"
        label="Add"
        tone="neutral"
        :disabled="disabled"
        @click="addValidation"
      />
    </div>
    <h2 class="p-3 pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Currently Validating:
    </h2>
    <ul class="flex flex-col p-3 gap-1 list-disc list-inside">
      <li
        v-for="protoView in prototypeViews"
        :key="protoView.key"
        class="flex flex-row gap-1 items-center text-sm pb-2 pl-4"
      >
        <div class="pr-2" role="decoration">•</div>
        {{ protoView.schemaVariantName }}: {{ protoView.propName }}
        <VButton
          class="flex-none"
          tone="neutral"
          variant="transparent"
          label=""
          icon="trash"
          :disabled="disabled"
          @click="deleteValidation(protoView.proto)"
        />
      </li>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import * as _ from "lodash-es";
import { VButton } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import {
  ValidationAssociations,
  ValidationPrototypeView,
} from "@/store/func/types";

const funcStore = useFuncStore();
const { schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: ValidationAssociations;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: ValidationAssociations): void;
  (e: "change", v: ValidationAssociations): void;
}>();

function nilId(): string {
  return "00000000000000000000000000";
}

const noneVariant = { label: "select schema variant", value: nilId() };
const noneProp = { label: "select attribute to validate", value: nilId() };

const selectedVariant = ref<Option>(noneVariant);
const selectedProp = ref<Option>(noneProp);

const propOptions = computed<Option[]>(() =>
  funcStore.propsAsOptionsForSchemaVariant(
    typeof selectedVariant.value.value === "string"
      ? selectedVariant.value.value
      : nilId(),
  ),
);

const prototypeViews = computed(() => {
  return props.modelValue.prototypes.map((proto) => {
    const schemaVariantName =
      schemaVariantOptions.value.find(
        (sv) => sv.value === proto.schemaVariantId,
      )?.label ?? "none";
    const propName = funcStore.propIdToSourceName(proto.propId) ?? "none";

    return {
      schemaVariantName,
      propName,
      key: `${proto.id}-${proto.schemaVariantId}`,
      proto: { ...proto },
    };
  });
});

const addValidation = () => {
  const prototypes = Array.from(
    new Set(
      props.modelValue.prototypes.concat({
        id: nilId(),
        schemaVariantId: selectedVariant.value.value as string,
        propId: selectedProp.value.value as string,
      }),
    ),
  );

  emit("update:modelValue", { type: "validation", prototypes });
  emit("change", { type: "validation", prototypes });
  selectedVariant.value = noneVariant;
  selectedProp.value = noneProp;
};

const deleteValidation = (protoToDelete: ValidationPrototypeView) => {
  const prototypes = props.modelValue.prototypes.filter(
    (proto) => !_.isEqual(proto, protoToDelete),
  );
  emit("update:modelValue", { type: "validation", prototypes });
  emit("change", { type: "validation", prototypes });
};
</script>
