<template>
  <div class="p-xs flex flex-col gap-xs w-full">
    <DropdownMenuButton
      placeholder="Pick an asset to manage"
      :disabled="props.disabled"
      :options="schemaOptions"
      @select="addSchema"
    />
    <ul v-if="binding?.managedSchemas && binding.managedSchemas.length > 0">
      <li
        v-for="schemaId in binding.managedSchemas"
        :key="schemaId"
        class="flex flex-row items-center content-center gap-2xs mt-1"
      >
        <TruncateWithTooltip
          class="grow text-neutral-700 type-bold-sm dark:text-neutral-50 ml-2"
        >
          {{ schemaNameMap[schemaId] ?? schemaId }}
        </TruncateWithTooltip>
        <IconButton
          icon="trash"
          size="sm"
          :disabled="props.disabled"
          iconTone="destructive"
          @click="removeSchema(schemaId)"
        />
      </li>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ref, watch, computed, toRaw } from "vue";
import { storeToRefs } from "pinia";
import {
  DropdownMenuButton,
  IconButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { Option } from "@/components/SelectMenu.vue";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { FuncId } from "@/api/sdf/dal/func";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { nonNullable } from "@/utils/typescriptLinter";

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const { schemaVariantOptions } = storeToRefs(assetStore);

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
  disabled?: boolean;
}>();

const schemaNameMap = ref<{ [key: string]: string }>({});

const managerSchemaId = computed(
  () =>
    assetStore.variantList.find(
      (variant) => variant.schemaVariantId === props.schemaVariantId,
    )?.schemaId,
);

const binding = computed(() => {
  const bindings = funcStore.managementBindings[props.funcId];
  const binding = bindings
    ?.filter((b) => b.schemaVariantId === props.schemaVariantId)
    .pop();
  return binding;
});

const schemaOptions = computed(() => {
  const uninstalled: Option[] = assetStore.uninstalledVariantList.map(
    (unin) => ({
      label: unin.displayName ?? unin.schemaName,
      value: unin.schemaId,
    }),
  );

  const installed = assetStore.variantList.map((inst) => ({
    label: inst.displayName ?? inst.schemaName,
    value: inst.schemaId,
  }));

  const currentManagedSchemas = binding.value?.managedSchemas ?? [];

  const allOptions = uninstalled.concat(installed);
  allOptions.forEach((option) => {
    const schemaId = option.value.toString();
    schemaNameMap.value[schemaId] = option.label;
  });

  const filteredOptions = _.uniq(
    allOptions.filter(
      (opt) =>
        typeof opt.value === "string" &&
        opt.value !== managerSchemaId.value &&
        !currentManagedSchemas.includes(opt.value),
    ),
  );
  filteredOptions.sort((a, b) => a.label.localeCompare(b.label));

  return filteredOptions;
});

const addSchema = async (selectedManagedSchemaId: string) => {
  if (binding.value) {
    const updated_binding = structuredClone(toRaw(binding.value));
    if (!updated_binding.managedSchemas) {
      updated_binding.managedSchemas = [];
    }
    updated_binding.managedSchemas.push(selectedManagedSchemaId);
    await funcStore.UPDATE_BINDING(props.funcId, [updated_binding]);
  }
};

const removeSchema = async (schemaId: string) => {
  if (binding.value) {
    const updated_binding = _.clone(binding.value);
    if (!updated_binding.managedSchemas) {
      updated_binding.managedSchemas = [];
    }
    updated_binding.managedSchemas = updated_binding.managedSchemas.filter(
      (id) => id !== schemaId,
    );
    await funcStore.UPDATE_BINDING(props.funcId, [updated_binding]);
  }
};

const validSchemaVariantIds = computed(() => {
  const bindings = funcStore.managementBindings[props.funcId];
  return bindings?.map((b) => b.schemaVariantId).filter(nonNullable);
});

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []),
);

watch(
  [validSchemaVariantIds, schemaVariantOptions],
  () => {
    selectedVariants.value = toOptionValues(
      schemaVariantOptions.value,
      validSchemaVariantIds.value || [],
    );
  },
  { immediate: true },
);

const detachFunc = () => {
  if (binding.value) funcStore.DELETE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
