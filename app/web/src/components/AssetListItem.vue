<template>
  <TreeNode
    :color="a.color"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    :isSelected="selectedAssetId === a.id"
    showSelection
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
        {{ assetNameString(a) }}
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
  ListedVariant,
} from "@/store/asset.store";

const props = defineProps({
  a: { type: Object as PropType<AssetListEntry>, required: true },
  c: { type: Array<ListedVariant> },
});

const route = useRoute();
const router = useRouter();
const assetStore = useAssetStore();
const { selectedAssetId } = storeToRefs(assetStore);

const assetNameString = (a: AssetListEntry) => {
  const name = assetDisplayName(a);
  if (!props.c) return name;

  const duplicates = props.c.filter(
    (asset) => assetDisplayName(asset) === name,
  );
  if (duplicates.length > 1) {
    return `${name} (${duplicates.indexOf(a)})`;
  } else return name;
};
</script>
