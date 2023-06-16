<template>
  <div>
    <div class="p-3 flex flex-col gap-2">
      <template v-if="!schemaVariantId">
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
      </template>
      <h2
        v-if="schemaVariantId"
        class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50"
      >
        Validate Attribute:
      </h2>

      <SelectMenu
        v-model="selectedProp"
        class="flex-auto"
        :options="propOptions"
      />
      <VButton
        v-if="!schemaVariantId"
        icon="plus"
        label="Add"
        tone="neutral"
        :disabled="disabled"
        @click="onClickAdd"
      />
    </div>
    <template v-if="!schemaVariantId">
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
            @click="onClickDelete(protoView.proto)"
          />
        </li>
      </ul>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import * as _ from "lodash-es";
import { VButton } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import {
  FuncAssociations,
  ValidationAssociations,
  ValidationPrototypeView,
} from "@/store/func/types";

const funcStore = useFuncStore();
const { schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: ValidationAssociations;
  schemaVariantId?: string;
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

const selectedVariant = ref<Option>(
  props.schemaVariantId
    ? { label: "", value: props.schemaVariantId }
    : noneVariant,
);

const propOptions = computed<Option[]>(() =>
  funcStore.propsAsOptionsForSchemaVariant(
    typeof selectedVariant.value.value === "string"
      ? selectedVariant.value.value
      : nilId(),
  ),
);

const selectedProp = ref<Option>(noneProp);

watch(
  [propOptions, () => props.schemaVariantId],
  () => {
    if (props.schemaVariantId) {
      const currentlyValidating = props.modelValue.prototypes.find(
        (proto) => proto.schemaVariantId === props.schemaVariantId,
      );
      if (currentlyValidating) {
        selectedProp.value =
          propOptions.value.find(
            (opt) => opt.value === currentlyValidating.propId,
          ) ?? noneProp;
      }
    }
  },
  { immediate: true },
);

watch(selectedProp, (newProp, oldProp) => {
  if (props.schemaVariantId) {
    let prototypes = props.modelValue.prototypes;
    const currentlyValidating = prototypes.find(
      (proto) => proto.schemaVariantId === props.schemaVariantId,
    );
    if (newProp.value === currentlyValidating?.propId) {
      // nothing to do.
      return;
    }
    if (oldProp.value === currentlyValidating?.propId) {
      prototypes = deleteValidation(prototypes, currentlyValidating);
    }
    prototypes = addValidation(
      prototypes,
      props.schemaVariantId,
      selectedProp.value.value as string,
    );
    emit("update:modelValue", { type: "validation", prototypes });
    emit("change", { type: "validation", prototypes });
  }
});

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

const onClickAdd = () => {
  const prototypes = addValidation(
    props.modelValue.prototypes,
    selectedVariant.value.value as string,
    selectedProp.value.value as string,
  );
  emit("update:modelValue", { type: "validation", prototypes });
  emit("change", { type: "validation", prototypes });
  selectedVariant.value = noneVariant;
  selectedProp.value = noneProp;
};

const addValidation = (
  prototypes: ValidationPrototypeView[],
  schemaVariantId: string,
  propId: string,
) => {
  return Array.from(
    new Set(
      prototypes.concat({
        id: nilId(),
        schemaVariantId,
        propId,
      }),
    ),
  );
};

const onClickDelete = (protoToDelete: ValidationPrototypeView) => {
  const prototypes = deleteValidation(
    props.modelValue.prototypes,
    protoToDelete,
  );
  emit("update:modelValue", { type: "validation", prototypes });
  emit("change", { type: "validation", prototypes });
};

const deleteValidation = (
  prototypes: ValidationPrototypeView[],
  protoToDelete: ValidationPrototypeView,
) => prototypes.filter((proto) => !_.isEqual(proto, protoToDelete));

const detachFunc = (): FuncAssociations | undefined => {
  if (props.schemaVariantId) {
    const prototypes = props.modelValue.prototypes;
    const currentlyValidating = props.modelValue.prototypes.find(
      (proto) => proto.schemaVariantId === props.schemaVariantId,
    );
    if (currentlyValidating) {
      return {
        type: "validation",
        prototypes: deleteValidation(prototypes, currentlyValidating),
      };
    }
  }
};

defineExpose({ detachFunc });
</script>
