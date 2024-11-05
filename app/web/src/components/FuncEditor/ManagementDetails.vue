<template>
  <div class="p-xs flex flex-col gap-xs w-full">
    <h2 class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      Managed assets
    </h2>

    <div class="flex flex-row gap-2xs">
      <SelectMenu
        v-model="selectedManagedSchemaId"
        label="Pick an asset to manage"
        type="dropdown"
        :disabled="props.disabled"
        :options="schemaOptions"
        class="grow"
      />

      <VButton
        icon="plus"
        size="sm"
        tone="success"
        :requestStatus="updateBindingReqStatus"
        :disabled="props.disabled"
        @click="addSchema"
      />
    </div>

    <ul v-if="binding?.managedSchemas">
      <li
        v-for="schemaId in binding.managedSchemas"
        :key="schemaId"
        class="flex flex-row items-center content-center gap-2xs mt-1"
      >
        <p class="grow text-neutral-700 type-bold-sm dark:text-neutral-50 ml-2">
          {{ schemaNameMap[schemaId] ?? schemaId }}
        </p>
        <VButton
          icon="trash"
          size="sm"
          :disabled="props.disabled"
          tone="destructive"
          @click="removeSchema(schemaId)"
        />
      </li>
    </ul>

    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50 mt-5">
      <p class="text-sm">You can detach this function above.</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ref, watch, computed } from "vue";
import { storeToRefs } from "pinia";
import { VButton } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { FuncId } from "@/api/sdf/dal/func";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { nonNullable } from "@/utils/typescriptLinter";
import { nilId } from "@/utils/nilId";

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const { schemaVariantOptions } = storeToRefs(assetStore);

const updateBindingReqStatus = funcStore.getRequestStatus("UPDATE_BINDING");

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
  disabled?: boolean;
}>();

const noneOutput = {
  label: "select asset to manage",
  value: nilId(),
};
const selectedManagedSchemaId = ref<Option>(noneOutput);

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

const addSchema = async () => {
  if (binding.value) {
    const updated_binding = _.clone(binding.value);
    if (!updated_binding.managedSchemas) {
      updated_binding.managedSchemas = [];
    }
    updated_binding.managedSchemas.push(
      selectedManagedSchemaId.value.value.toString(),
    );
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
