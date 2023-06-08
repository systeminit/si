<template>
  <div>
    <ScrollArea v-if="assetStore.selectedAssetId">
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
          <SiFuncListItem :func="func" color="#921ed6" />
        </li>
      </ul>
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import { ScrollArea } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";

const assetStore = useAssetStore();

defineProps<{
  assetId?: string;
}>();
</script>
