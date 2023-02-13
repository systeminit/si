<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :request-status="loadAssetReqStatus"
      show-loader-without-message
    />
    <div v-else-if="assetStore.selectedAsset && assetId" class="flex flex-col">
      <div
        class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
      >
        <NodeSkeleton :color="assetStore.selectedAsset.color" size="mini" />
        <div class="font-bold truncate leading-relaxed">
          {{ assetDisplayName(assetStore.selectedAsset) }}
        </div>
        <VButton2
          label="Execute"
          :disabled="disabled"
          tone="action"
          icon="plus"
          size="md"
          @click="executeAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <SiTextBox
          id="name"
          v-model="assetStore.selectedAsset.name"
          :disabled="disabled"
          title="Name"
          placeholder="Give this asset a name here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <SiTextBox
          id="menuName"
          v-model="assetStore.selectedAsset.menuName"
          :disabled="disabled"
          title="Display name"
          placeholder="Optionally, give the asset a shorter name for display here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <SiTextBox
          id="category"
          v-model="assetStore.selectedAsset.category"
          :disabled="disabled"
          title="Category"
          placeholder="Pick a category for this asset"
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <SiTextBox
          id="description"
          v-model="assetStore.selectedAsset.description"
          :disabled="disabled"
          title="Description"
          text-area
          placeholder="Provide a brief description of this asset here..."
          @blur="updateAsset"
        />
      </div>
      <div class="p-sm flex items-center">
        <SiTextBox
          id="color"
          v-model="assetStore.selectedAsset.color"
          :disabled="disabled"
          title="Color"
          placeholder="Choose a color for this asset"
          @blur="updateAsset"
        />
        <div
          class="box-border h-8 w-8 mt-[23px] ml-auto"
          :style="`background-color: #${assetStore.selectedAsset.color}`"
        ></div>
      </div>
      <div class="p-sm flex flex-col">
        <SiTextBox
          id="link"
          v-model="assetStore.selectedAsset.link"
          :disabled="disabled"
          title="Documentation Link"
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
import VButton2 from "@/ui-lib/VButton2.vue";
import { useAssetStore, assetDisplayName } from "@/store/asset.store";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import Modal from "@/ui-lib/modals/Modal.vue";
import SiTextBox from "@/components/SiTextBox.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const assetStore = useAssetStore();
const loadAssetReqStatus = assetStore.getRequestStatus("LOAD_ASSET");
const executeAssetModalRef = ref();
const assetModalTitle = ref("New Asset Created");

const updateAsset = () => {
  assetStore.SAVE_ASSET(assetStore.selectedAsset);
};

const disabled = computed(() => assetStore.selectedAsset.variantExists);

defineProps<{
  assetId?: string;
}>();

const executeAsset = async () => {
  await assetStore.EXEC_ASSET(assetStore.selectedAsset.id);
  executeAssetModalRef.value.open();
};
</script>
