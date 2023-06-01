<template>
  <ScrollArea>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending"
      :request-status="loadAssetsReqStatus"
      loading-message="Loading assets..."
    />
    <template v-if="loadAssetsReqStatus.isSuccess" #top>
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <!-- TODO - currently this button doesn't do anything -->
        <VButton
          label="Author New Asset"
          tone="action"
          icon="plus"
          size="sm"
          @click="newAsset"
        />
      </div>
      <SiSearch auto-search placeholder="search assets" />
      <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
      >
        Select an asset to view or edit it.
      </div>
    </template>
    <template v-if="loadAssetsReqStatus.isSuccess">
      <ul class="overflow-y-auto min-h-[200px]">
        <AssetListItem v-for="a in assetStore.assetList" :key="a.id" :a="a" />
      </ul>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { onMounted } from "vue";
import {
  ScrollArea,
  VButton,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import SiSearch from "@/components/SiSearch.vue";
import { useAssetStore } from "@/store/asset.store";
import AssetListItem from "./AssetListItem.vue";

const assetStore = useAssetStore();
const router = useRouter();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const props = defineProps({
  assetId: { type: String },
});

onMounted(() => {
  if (!props.assetId) {
    assetStore.SELECT_ASSET(null);
  }
});

const newAsset = async () => {
  const result = await assetStore.CREATE_ASSET(assetStore.createNewAsset());
  if (result.result.success) {
    assetStore.SELECT_ASSET(result.result.data.id);
    router.push({
      name: "workspace-lab-assets",
      params: {
        ...router.currentRoute.value.params,
        assetId: result.result.data.id,
      },
    });
  }
};
</script>
