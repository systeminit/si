<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :request-status="loadAssetReqStatus"
      show-loader-without-message
    />

    <ScrollArea
      v-if="assetStore.selectedAssetId && !loadAssetReqStatus.isPending"
    >
      <template #top>
        <SidebarSubpanelTitle class="border-t-0 text-center">
          Asset Functions
        </SidebarSubpanelTitle>
      </template>

      <ul class="overflow-y-auto min-h-[200px]">
        <li
          v-for="func in assetStore.assetsById[assetStore.selectedAssetId]
            ?.funcs ?? []"
          :key="func.id"
        >
          <SiFuncListItem
            :func="func"
            color="#921ed6"
            context="workspace-lab-assets"
            :selected-func-id="funcStore.selectedFuncId"
          />
        </li>
      </ul>
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import { ScrollArea, RequestStatusMessage } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";

const props = defineProps<{
  assetId?: string;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
);
</script>
