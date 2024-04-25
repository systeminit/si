<template>
  <TreeNode
    :color="a.color"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
        selectedAssetId === a.id
          ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300 py-0'
          : 'dark:hover:text-action-300 hover:text-action-500',
      )
    "
    @mousedown.left.stop="
      router.push({
        name: 'workspace-lab-assets',
        params: {
          ...route.params,
          assetId: a.id,
          funcId: undefined,
        },
      })
    "
    @click.right.prevent
  >
    <template #label>
      <div class="text-xs w-full truncate">
        {{ assetDisplayName(a) }}
      </div>
    </template>
  </TreeNode>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { TreeNode } from "@si/vue-lib/design-system";
import clsx from "clsx";
import {
  AssetListEntry,
  useAssetStore,
  assetDisplayName,
} from "@/store/asset.store";

defineProps({
  a: { type: Object as PropType<AssetListEntry>, required: true },
});

const route = useRoute();
const router = useRouter();
const assetStore = useAssetStore();
const { selectedAssetId } = storeToRefs(assetStore);
</script>
