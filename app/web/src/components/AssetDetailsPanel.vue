<template>
  <div class="grow relative">
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
    />
    <ScrollArea v-else-if="editingAsset && assetId">
      <template #top>
        <div
          class="flex flex-row items-center gap-2 p-xs border-b dark:border-neutral-600"
        >
          <VButton
            :requestStatus="executeAssetReqStatus"
            loadingText="Creating Asset..."
            :label="
              editingAsset.schemaVariantId ? 'Update Asset' : 'Create Asset'
            "
            :disabled="disabled"
            tone="action"
            icon="bolt"
            size="md"
            @click="executeAsset"
          />
          <VButton
            label="Clone"
            tone="neutral"
            icon="clipboard-copy"
            size="md"
            @click="cloneAsset"
          />
        </div>
      </template>

      <Stack class="p-xs py-sm">
        <ErrorMessage v-if="disabled" icon="alert-triangle" tone="warning"
          >{{ disabledWarning }}
        </ErrorMessage>

        <ErrorMessage
          v-if="executeAssetReqStatus.isError"
          :requestStatus="executeAssetReqStatus"
        />
        <VormInput
          id="name"
          v-model="editingAsset.name"
          type="text"
          :disabled="disabled"
          label="Name"
          placeholder="(mandatory) Provide the asset a name"
          @blur="updateAsset"
        />
        <VormInput
          id="menuName"
          v-model="editingAsset.menuName"
          type="text"
          :disabled="disabled"
          label="Display name"
          placeholder="(optional) Provide the asset a shorter display name"
          @blur="updateAsset"
        />
        <VormInput
          id="handler"
          v-model="editingAsset.handler"
          type="text"
          :disabled="disabled"
          label="Entrypoint"
          placeholder="(mandatory) Provide the function entrypoint to the asset"
          @blur="updateAsset"
        />
        <VormInput
          id="category"
          v-model="editingAsset.category"
          type="text"
          :disabled="disabled"
          label="Category"
          placeholder="(mandatory) Provide a category for the asset"
          @blur="updateAsset"
        />
        <VormInput
          id="componentType"
          v-model="editingAsset.componentType"
          type="dropdown"
          :disabled="disabled"
          :options="componentTypeOptions"
          label="Component Type"
          @change="updateAsset"
        />
        <VormInput
          id="description"
          v-model="editingAsset.description"
          type="textarea"
          :disabled="disabled"
          label="Description"
          placeholder="(optional) Provide a brief description of the asset"
          @blur="updateAsset"
        />
        <VormInput type="container" label="color" :disabled="disabled">
          <ColorPicker
            id="color"
            v-model="editingAsset.color"
            :disabled="disabled"
            @change="updateAsset"
          />
        </VormInput>

        <VormInput
          id="link"
          v-model="editingAsset.link"
          type="url"
          :disabled="disabled"
          label="Documentation Link"
          placeholder="(optional) Provide a documentation link for the asset"
          @blur="updateAsset"
        />
      </Stack>
    </ScrollArea>
    <div
      v-else
      class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
    >
      <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
      <template v-else>Select an asset to view its details.</template>
    </div>
    <Modal ref="executeAssetModalRef" size="sm" :title="assetModalTitle">
      The asset you just created will now appear in the Assets Panel.
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import {
  VButton,
  VormInput,
  RequestStatusMessage,
  Modal,
  ErrorMessage,
  Stack,
  ScrollArea,
} from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { nilId } from "@/utils/nilId";
import { useComponentsStore } from "@/store/components.store";
import ColorPicker from "./ColorPicker.vue";

const props = defineProps<{
  assetId?: string;
}>();

const componentsStore = useComponentsStore();
const assetStore = useAssetStore();
const funcStore = useFuncStore();
const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
);
const executeAssetReqStatus = assetStore.getRequestStatus(
  "EXEC_ASSET",
  props.assetId,
);
const executeAssetModalRef = ref();
const assetModalTitle = ref("New Asset Created");

const componentTypeOptions = [
  { label: "Aggregation Frame", value: "aggregationFrame" },
  { label: "Component", value: "component" },
  { label: "Configuration Frame", value: "configurationFrame" },
];

const editingAsset = ref(_.cloneDeep(assetStore.selectedAsset));
watch(
  () => assetStore.selectedAsset,
  () => {
    editingAsset.value = _.cloneDeep(assetStore.selectedAsset);
  },
);

const updateAsset = async () => {
  if (
    editingAsset.value &&
    !_.isEqual(editingAsset.value, assetStore.selectedAsset)
  ) {
    await assetStore.SAVE_ASSET(editingAsset.value);
  }
};

const disabled = computed(
  () =>
    !!(
      (editingAsset.value?.hasComponents || editingAsset.value?.hasAttrFuncs) ??
      false
    ),
);

const disabledWarning = computed(() => {
  let byComponents = "";
  if (editingAsset.value?.hasComponents) {
    byComponents = "by components";
  }
  let byFuncs = "";
  if (editingAsset.value?.hasAttrFuncs) {
    byFuncs = "by attribute functions or custom validations";
  }
  const and =
    editingAsset.value?.hasComponents && editingAsset.value?.hasAttrFuncs
      ? " and "
      : "";

  return `This asset cannot be edited because it is in use ${byComponents}${and}${byFuncs}.`;
});

const executeAsset = async () => {
  if (assetStore.selectedAssetId) {
    const result = await assetStore.EXEC_ASSET(assetStore.selectedAssetId);
    if (result.result.success) {
      executeAssetModalRef.value?.open();
      const { schemaVariantId } = result.result.data;
      if (schemaVariantId !== nilId()) {
        assetStore.setSchemaVariantIdForAsset(
          assetStore.selectedAssetId,
          schemaVariantId,
        );
        // We need to reload both schemas and assets since they're stored separately
        await assetStore.LOAD_ASSET(assetStore.selectedAssetId);
        await componentsStore.FETCH_AVAILABLE_SCHEMAS();
        await funcStore.FETCH_INPUT_SOURCE_LIST(schemaVariantId); // a new asset means new input sources
      }
    }
  }
};

const cloneAsset = async () => {
  if (editingAsset.value?.id) {
    const result = await assetStore.CLONE_ASSET(editingAsset.value.id);
    if (result.result.success) {
      assetStore.selectAsset(result.result.data.id);
    }
  }
};
</script>
