<template>
  <RouterLink
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      selectedAssetId === a.id
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: 'workspace-lab-assets',
      params: { ...route.params, assetId: a.id },
    }"
  >
    <NodeSkeleton :color="`${a.color}`" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ assetDisplayName(a) }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { useRoute } from "vue-router";
import { storeToRefs } from "pinia";
import {
  AssetListEntry,
  useAssetStore,
  assetDisplayName,
} from "@/store/asset.store";
import NodeSkeleton from "./NodeSkeleton.vue";

defineProps({
  a: { type: Object as PropType<AssetListEntry>, required: true },
});

const route = useRoute();
const assetStore = useAssetStore();
const { selectedAssetId } = storeToRefs(assetStore);
</script>
