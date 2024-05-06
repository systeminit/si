<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
    />
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle
          label="Asset Functions"
          icon="func"
          class="mt-2xs"
        >
          <AssetFuncAttachDropdown
            v-if="assetStore.selectedAssetId"
            :disabled="!assetStore.selectedAsset?.id"
            @selected-attach-type="openAttachFuncModal"
          />
        </SidebarSubpanelTitle>
      </template>

      <FuncList
        v-if="assetStore.selectedAssetId && !loadAssetReqStatus.isPending"
        :funcsByKind="funcsByKind"
        context="workspace-lab-assets"
        defaultOpen
      />
      <EmptyStateCard
        v-else
        iconName="funcs"
        primaryText="Select Asset, View Functions"
        secondaryText="Select an asset from the list above to view its attached functions here."
      />
    </ScrollArea>
    <AssetFuncAttachModal
      ref="attachModalRef"
      :schemaVariantId="assetSchemaVariantId"
      :assetId="assetId"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import groupBy from "lodash-es/groupBy";
import { RequestStatusMessage, ScrollArea } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetFuncAttachDropdown from "./AssetFuncAttachDropdown.vue";
import FuncList from "./FuncEditor/FuncList.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

const props = defineProps<{ assetId?: string }>();

const assetStore = useAssetStore();

const funcsByKind = computed(() =>
  props.assetId
    ? groupBy(assetStore.assetsById[props.assetId]?.funcs ?? [], (f) => f.kind)
    : {},
);

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
);

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();
const assetSchemaVariantId = computed(() =>
  props.assetId ? assetStore.assetsById[props.assetId]?.id : undefined,
);

const openAttachFuncModal = (type: "new" | "existing") => {
  if (type === "new") {
    attachModalRef.value?.open(false);
  } else {
    attachModalRef.value?.open(true);
  }
};
</script>
