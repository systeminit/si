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
        <SiCollapsible
          v-for="category in Object.keys(categorizedAssets)"
          :key="category"
          :label="category"
          as="li"
          content-as="ul"
          default-open
          class="select-none"
        >
          <AssetListItem
            v-for="asset in categorizedAssets[category]"
            :key="asset.id"
            :a="asset"
          />
        </SiCollapsible>
      </ul>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import { onMounted, computed } from "vue";
import { storeToRefs } from "pinia";
import {
  ScrollArea,
  VButton,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import SiSearch from "@/components/SiSearch.vue";
import { AssetListEntry, useAssetStore } from "@/store/asset.store";
import AssetListItem from "./AssetListItem.vue";
import SiCollapsible from "./SiCollapsible.vue";

const assetStore = useAssetStore();
const { assetList } = storeToRefs(assetStore);
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

const categorizedAssets = computed(() =>
  assetList.value.reduce((categorized, asset) => {
    let catList = categorized[asset.category];
    if (!catList) {
      catList = [];
    }
    catList.push(asset);
    categorized[asset.category] = catList;
    return categorized;
  }, {} as { [key: string]: AssetListEntry[] }),
);

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
