<template>
  <div class="grow relative">
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
      showLoaderWithoutMessage
    />
    <ScrollArea v-else-if="assetStore.selectedAsset && assetId">
      <template #top>
        <div
          v-if="!changeSetsStore.headSelected"
          class="flex flex-row items-center gap-2 p-xs border-b dark:border-neutral-600"
        >
          <VButton
            :requestStatus="executeAssetReqStatus"
            loadingText="Creating Asset..."
            :label="
              assetStore.selectedAsset.schemaVariantId
                ? 'Update Asset'
                : 'Create Asset'
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
          v-model="assetStore.selectedAsset.name"
          type="text"
          :disabled="disabled"
          label="Name"
          placeholder="Give this asset a name here..."
          @blur="updateAsset"
        />
        <VormInput
          id="menuName"
          v-model="assetStore.selectedAsset.menuName"
          type="text"
          :disabled="disabled"
          label="Display name"
          placeholder="Optionally, give the asset a shorter name for display here..."
          @blur="updateAsset"
        />
        <VormInput
          id="handler"
          v-model="assetStore.selectedAsset.handler"
          type="text"
          :disabled="disabled"
          label="Entrypoint"
          placeholder="Optionally, give the asset a shorter name for display here..."
          @blur="updateAsset"
        />
        <VormInput
          id="category"
          v-model="assetStore.selectedAsset.category"
          type="text"
          :disabled="disabled"
          label="Category"
          placeholder="Pick a category for this asset"
          @blur="updateAsset"
        />
        <VormInput
          id="componentType"
          v-model="assetStore.selectedAsset.componentType"
          type="dropdown"
          :disabled="disabled"
          :options="componentTypeOptions"
          label="Component Type"
          @change="updateAsset"
        />
        <VormInput
          id="description"
          v-model="assetStore.selectedAsset.description"
          type="textarea"
          :disabled="disabled"
          label="Description"
          placeholder="Provide a brief description of this asset here..."
          @blur="updateAsset"
        />
        <VormInput type="container" label="color" :disabled="disabled">
          <ColorPicker
            id="color"
            v-model="assetStore.selectedAsset.color"
            :disabled="disabled"
            @change="updateAsset"
          />
        </VormInput>

        <VormInput
          id="link"
          v-model="assetStore.selectedAsset.link"
          type="url"
          :disabled="disabled"
          label="Documentation Link"
          placeholder="Enter a link to the documentation for this asset here..."
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
import { computed, ref } from "vue";
import {
  VButton,
  VormInput,
  RequestStatusMessage,
  Modal,
  ErrorMessage,
  Stack,
  ScrollArea,
} from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { nilId } from "@/utils/nilId";
import ColorPicker from "./ColorPicker.vue";

defineProps<{
  assetId?: string;
}>();

const changeSetsStore = useChangeSetsStore();
const assetStore = useAssetStore();
const funcStore = useFuncStore();
const loadAssetReqStatus = assetStore.getRequestStatus("LOAD_ASSET");
const executeAssetModalRef = ref();
const assetModalTitle = ref("New Asset Created");

const componentTypeOptions = [
  { label: "Aggregation Frame", value: "aggregationFrame" },
  { label: "Component", value: "component" },
  { label: "Configuration Frame", value: "configurationFrame" },
];

const updateAsset = () => {
  if (assetStore.selectedAsset) {
    assetStore.SAVE_ASSET(assetStore.selectedAsset);
  }
};

const disabled = computed(
  () =>
    !!(
      (assetStore.selectedAsset?.hasComponents ||
        assetStore.selectedAsset?.hasAttrFuncs) ??
      false
    ),
);

const disabledWarning = computed(() => {
  let byComponents = "";
  if (assetStore.selectedAsset?.hasComponents) {
    byComponents = "by components";
  }
  let byFuncs = "";
  if (assetStore.selectedAsset?.hasAttrFuncs) {
    byFuncs = "by attribute functions or custom validations";
  }
  const and =
    assetStore.selectedAsset?.hasComponents &&
    assetStore.selectedAsset?.hasAttrFuncs
      ? " and "
      : "";

  return `This asset cannot be edited because it is in use ${byComponents}${and}${byFuncs}.`;
});

const executeAsset = async () => {
  if (assetStore.selectedAssetId) {
    const result = await assetStore.EXEC_ASSET(assetStore.selectedAssetId);
    if (result.result.success) {
      executeAssetModalRef.value.open();
      const { schemaVariantId } = result.result.data;
      if (schemaVariantId !== nilId()) {
        assetStore.setSchemaVariantIdForAsset(
          assetStore.selectedAssetId,
          schemaVariantId,
        );
        await assetStore.LOAD_ASSET(schemaVariantId);
        await funcStore.FETCH_INPUT_SOURCE_LIST(schemaVariantId); // a new asset means new input sources
      }
    }
  }
};

const executeAssetReqStatus = assetStore.getRequestStatus(
  "EXEC_ASSET",
  assetStore.selectedAsset?.id,
);

const cloneAsset = async () => {
  if (assetStore.selectedAsset?.id) {
    const result = await assetStore.CLONE_ASSET(assetStore.selectedAsset.id);
    if (result.result.success) {
      assetStore.SELECT_ASSET(result.result.data.id);
    }
  }
};
</script>
