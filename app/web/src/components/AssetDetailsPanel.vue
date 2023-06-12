<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :request-status="loadAssetReqStatus"
      show-loader-without-message
    />
    <div v-else-if="assetStore.selectedAsset && assetId" class="flex flex-col">
      <div
        class="p-sm border-b dark:border-neutral-600 flex flex-row items-center gap-2"
      >
        <VButton
          label="Create Asset"
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
      <div class="p-2">
        <ErrorMessage
          v-if="executeAssetReqStatus.isError"
          :request-status="executeAssetReqStatus"
        />
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="name"
          v-model="assetStore.selectedAsset.name"
          type="text"
          :disabled="disabled"
          label="Name"
          placeholder="Give this asset a name here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="menuName"
          v-model="assetStore.selectedAsset.menuName"
          type="text"
          :disabled="disabled"
          label="Display name"
          placeholder="Optionally, give the asset a shorter name for display here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="category"
          v-model="assetStore.selectedAsset.category"
          type="text"
          :disabled="disabled"
          label="Category"
          placeholder="Pick a category for this asset"
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="componentType"
          v-model="assetStore.selectedAsset.componentType"
          type="dropdown"
          :disabled="disabled"
          :options="componentTypeOptions"
          label="Component Type"
          @change="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="description"
          v-model="assetStore.selectedAsset.description"
          type="textarea"
          :disabled="disabled"
          label="Description"
          placeholder="Provide a brief description of this asset here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm">
        <label class="pl-[1px] text-sm font-bold" for="color">Color</label>
        <div class="mt-1 block">
          <ColorPicker
            id="color"
            v-model="assetStore.selectedAsset.color"
            @change="updateAsset"
          />
        </div>
      </div>
      <div class="p-sm flex flex-col">
        <VormInput
          id="link"
          v-model="assetStore.selectedAsset.link"
          :disabled="disabled"
          label="Documentation Link"
          placeholder="Enter a link to the documentation for this asset here..."
          @blur="updateAsset"
        />
        <div class="text-md text-action-500 font-bold">
          <a :href="assetStore.selectedAsset.link" target="_blank">
            Documentation Link
          </a>
        </div>
      </div>
    </div>
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
} from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import ColorPicker from "./ColorPicker.vue";

const assetStore = useAssetStore();
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
  () => assetStore.selectedAsset?.variantExists ?? false,
);

defineProps<{
  assetId?: string;
}>();

const executeAsset = async () => {
  if (assetStore.selectedAsset?.id) {
    const result = await assetStore.EXEC_ASSET(assetStore.selectedAsset.id);
    if (result.result.success) {
      executeAssetModalRef.value.open();
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
