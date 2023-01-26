<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending"
      :request-status="loadAssetsReqStatus"
      loading-message="Loading assets..."
    />
    <template v-else-if="loadAssetsReqStatus.isSuccess">
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <!-- TODO - currently this button doesn't do anything -->
        <VButton2
          label="Add Asset"
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
      <ul class="overflow-y-auto min-h-[200px]">
        <AssetListItem v-for="a in assetStore.assetsById" :key="a.id" :a="a" />
      </ul>
    </template>
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { useRouter } from "vue-router";
import { onMounted } from "vue";
import SiSearch from "@/components/SiSearch.vue";
import { useAssetStore } from "@/store/asset.store";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import AssetListItem from "./AssetListItem.vue";

const router = useRouter();
const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSETS");

const props = defineProps({
  slug: { type: String },
});

onMounted(() => {
  if (!props.slug) {
    assetStore.setSelectedAssetId(null);
  }
});

const newAsset = () => {
  const asset = assetStore.createNewAsset();
  assetStore.setSelectedAssetId(asset.id);
  router.push({
    name: "workspace-lab-assets",
    params: { assetSlug: asset.slug },
  });
};
</script>
