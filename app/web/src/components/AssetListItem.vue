<template>
  <TreeNode
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    :color="a.color"
    :isSelected="selectedAssets.includes(a.schemaVariantId)"
    showSelection
    @mousedown.left.stop="onClick"
    @click.right.prevent
  >
    <template #label>
      <div class="text-xs w-full truncate flex flex-row items-center gap-1">
        <div class="truncate">
          {{ schemaVariantDisplayName(a) }}
          <template v-if="!a.canCreateNewComponents">
            <i>version: {{ a.version }}</i>
          </template>
        </div>

        <div class="ml-auto flex flex-none gap-xs shrink-0">
          <EditingPill v-if="!a.isLocked" :color="a.color" />
          <Icon
            v-if="a.canContribute"
            name="cloud-upload"
            size="xs"
            tone="action"
            tooltip="Contribute"
            tooltipPlacement="top"
            variant="simple"
          />
          <Icon
            v-if="canUpdate"
            name="code-deployed"
            size="xs"
            tone="action"
            tooltip="Update"
            tooltipPlacement="top"
            variant="simple"
          />
        </div>
      </div>
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import { PropType, computed } from "vue";
import { storeToRefs } from "pinia";
import { TreeNode, Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { useAssetStore, schemaVariantDisplayName } from "@/store/asset.store";
import { useModuleStore } from "@/store/module.store";
import EditingPill from "./EditingPill.vue";

const props = defineProps({
  a: { type: Object as PropType<SchemaVariant>, required: true },
  c: { type: Array<SchemaVariant> },
});

const assetStore = useAssetStore();
const moduleStore = useModuleStore();

const { selectedSchemaVariants: selectedAssets } = storeToRefs(assetStore);

const canUpdate = computed(
  () => !!moduleStore.upgradeableModules[props.a.schemaVariantId],
);

const onClick = (e: MouseEvent) => {
  if (e.shiftKey) assetStore.addSchemaVariantSelection(props.a.schemaVariantId);
  else assetStore.setSchemaVariantSelection(props.a.schemaVariantId);
};
</script>
