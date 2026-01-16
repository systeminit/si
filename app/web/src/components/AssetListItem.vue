<template>
  <TreeNode
    ref="treeNodeRef"
    :classes="
      clsx(
        themeClasses('text-shade-100', 'text-shade-0 bg-neutral-800'),
        'py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    :color="a.color"
    :isSelected="isSelected"
    showSelection
    @mousedown.left.stop="onClick"
    @click.right.prevent
  >
    <template #label>
      <div class="text-xs w-full truncate flex flex-row items-center gap-2xs h-[20px]">
        <div class="truncate">
          {{ schemaVariantDisplayName(a) }}
          <template v-if="!a.canCreateNewComponents">
            <i>version: {{ a.version }}</i>
          </template>
        </div>

        <div class="ml-auto flex flex-none gap-xs shrink-0">
          <EditingPill v-if="!a.isLocked" :color="a.color" />
          <Icon
            v-if="canContribute"
            name="cloud-upload"
            size="xs"
            tone="action"
            tooltip="Contribute"
            tooltipPlacement="top"
          />
          <Icon v-if="canUpdate" name="code-deployed" size="xs" tone="action" tooltip="Update" tooltipPlacement="top" />
        </div>
      </div>
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import { PropType, computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { TreeNode, Icon, themeClasses } from "@si/vue-lib/design-system";
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

const isSelected = computed(() => selectedAssets.value.includes(props.a.schemaVariantId));

const canUpdate = computed(() => !!moduleStore.upgradeableModules[props.a.schemaVariantId]);

const canContribute = computed(() => {
  return moduleStore.contributableModules.includes(props.a.schemaVariantId) || props.a.canContribute;
});

const onClick = (e: MouseEvent) => {
  if (e.shiftKey) {
    if (isSelected.value) {
      assetStore.removeSchemaVariantSelection(props.a.schemaVariantId);
    } else {
      assetStore.addSchemaVariantSelection(props.a.schemaVariantId);
    }
  } else assetStore.setSchemaVariantSelection(props.a.schemaVariantId);
};

const treeNodeRef = ref<InstanceType<typeof TreeNode>>();

const scrollIntoView = () => {
  if (treeNodeRef.value) {
    treeNodeRef.value.scrollIntoView();
  }
};

watch(
  () => isSelected.value,
  () => {
    if (isSelected.value) {
      scrollIntoView();
    }
  },
);
</script>
