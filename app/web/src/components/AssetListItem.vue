<template>
  <TreeNode
    :color="a.color"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    :isSelected="selectedAssets.includes(a.schemaVariantId)"
    showSelection
    @mousedown.left.stop="onClick"
    @click.right.prevent
  >
    <template #label>
      <div class="text-xs w-full truncate flex flex-row items-center gap-1">
        <div class="shrink-0">{{ assetNameString(a) }}</div>

        <div class="ml-auto flex flex-none gap-xs">
          <Icon
            v-if="a.canContribute"
            name="cloud-upload"
            variant="simple"
            tone="action"
            tooltip="Contribute"
            tooltipPlacement="top"
            size="xs"
          />
          <Icon
            v-if="a.canUpdate"
            name="code-deployed"
            variant="simple"
            tone="action"
            tooltip="Update"
            tooltipPlacement="top"
            size="xs"
          />
        </div>
      </div>
    </template>
  </TreeNode>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { storeToRefs } from "pinia";
import { TreeNode, Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import {
  SchemaVariantListEntry,
  useAssetStore,
  schemaVariantDisplayName,
} from "@/store/asset.store";

const props = defineProps({
  a: { type: Object as PropType<SchemaVariantListEntry>, required: true },
  c: { type: Array<SchemaVariantListEntry> },
});

const assetStore = useAssetStore();
const { selectedSchemaVariants: selectedAssets } = storeToRefs(assetStore);

const onClick = (e: MouseEvent) => {
  if (e.shiftKey) assetStore.addSchemaVariantSelection(props.a.schemaVariantId);
  else assetStore.setSchemaVariantSelection(props.a.schemaVariantId);
};

const assetNameString = (a: SchemaVariantListEntry) => {
  const name = schemaVariantDisplayName(a);
  if (!props.c) return name;

  const duplicates = props.c.filter(
    (asset) => schemaVariantDisplayName(asset) === name,
  );
  if (duplicates.length > 1) {
    return `${name} (${duplicates.indexOf(a)})`;
  } else return name;
};
</script>
