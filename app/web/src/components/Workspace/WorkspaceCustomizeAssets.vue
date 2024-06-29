<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <component
    :is="ResizablePanel"
    ref="leftResizablePanelRef"
    rememberSizeKey="func-picker"
    side="left"
    :minSize="300"
  >
    <template #subpanel1>
      <div class="flex flex-col h-full">
        <div class="relative flex-grow">
          <CustomizeTabs tabContentSlug="assets">
            <AssetListPanel />
          </CustomizeTabs>
        </div>
      </div>
    </template>
    <template #subpanel2>
      <AssetFuncListPanel :assetId="assetStore.selectedVariantId" />
    </template>
  </component>

  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-0 bottom-2 absolute">
      <AssetEditorTabs />
    </div>
  </div>

  <component
    :is="ResizablePanel"
    ref="rightResizablePanelRef"
    rememberSizeKey="func-details"
    side="right"
    :minSize="300"
  >
    <div class="absolute w-full flex flex-col h-full">
      <AssetCard
        v-if="assetStore.selectedVariantId"
        titleCard
        :assetId="assetStore.selectedVariantId"
      />
      <template v-if="assetStore.selectedVariantId">
        <FuncDetails
          v-if="
            funcStore.selectedFuncId &&
            assetStore.selectedSchemaVariant?.schemaVariantId
          "
          :funcId="funcStore.selectedFuncId"
          :schemaVariantId="assetStore.selectedSchemaVariant.schemaVariantId"
          singleModelScreen
          allowTestPanel
          @expand-panel="rightResizablePanelRef?.maximize()"
        />
        <!-- the key here is to force remounting so we get the proper asset
        request statuses -->
        <AssetDetailsPanel
          v-else
          :key="assetStore.selectedVariantId"
          :assetId="assetStore.selectedVariantId"
        />
      </template>
      <template v-else-if="assetStore.selectedSchemaVariants.length > 1">
        <div class="flex flex-col h-full w-full overflow-hidden">
          <ScrollArea>
            <template #top>
              <SidebarSubpanelTitle label="Multiple Assets" icon="multiselect">
                <DetailsPanelMenuIcon @click="open" />
              </SidebarSubpanelTitle>
              <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
            </template>

            <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
              {{ assetStore.selectedSchemaVariants.length }} assets selected:
            </div>
            <Stack spacing="xs" class="p-xs">
              <AssetCard
                v-for="assetId in assetStore.selectedSchemaVariants"
                :key="assetId"
                :titleCard="false"
                :assetId="assetId"
              />
            </Stack>
          </ScrollArea>
        </div>
      </template>
      <EmptyStateCard
        v-else
        iconName="no-assets"
        primaryText="No Assets Selected"
        secondaryText="Select an asset from the list on the left panel to view its details here."
      />
    </div>
  </component>
</template>

<script lang="ts" setup>
import { onBeforeUnmount, onMounted, ref, computed, watch } from "vue";
import {
  ResizablePanel,
  ScrollArea,
  Stack,
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import AssetCard from "../AssetCard.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditorTabs from "../AssetEditorTabs.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";
import EmptyStateCard from "../EmptyStateCard.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";
import DetailsPanelMenuIcon from "../DetailsPanelMenuIcon.vue";

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const leftResizablePanelRef = ref<InstanceType<typeof ResizablePanel>>();
const rightResizablePanelRef = ref<InstanceType<typeof ResizablePanel>>();

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const open = (mouse: MouseEvent) => {
  contextMenuRef.value?.open(mouse, false);
};

const rightClickMenuItems = computed(() => {
  const canContribute = [];
  const canUpdate = [];
  assetStore.selectedSchemaVariantRecords.forEach((asset) => {
    if (asset.canContribute) canContribute.push(asset);
    if (asset.canUpdate) canUpdate.push(asset);
  });

  const items: DropdownMenuItemObjectDef[] = [
    {
      label: `Contribute ${
        canContribute.length ? canContribute.length : ""
      } Assets`,
      icon: "cloud-upload",
      onSelect: () => {}, // TODO
      disabled: canContribute.length === 0,
    },
    {
      label: `Update ${canUpdate.length ? canUpdate.length : ""} Assets`,
      icon: "code-deployed",
      onSelect: () => {}, // TODO
      disabled: canUpdate.length === 0,
    },
  ];
  return items;
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.altKey &&
    e.shiftKey &&
    leftResizablePanelRef.value &&
    rightResizablePanelRef.value
  ) {
    if (
      leftResizablePanelRef.value.collapsed &&
      rightResizablePanelRef.value.collapsed
    ) {
      // Open all panels
      leftResizablePanelRef.value.collapseSet(false);
      rightResizablePanelRef.value.collapseSet(false);
      leftResizablePanelRef.value.subpanelCollapseSet(false);
    } else {
      // Close all panels
      leftResizablePanelRef.value.collapseSet(true);
      rightResizablePanelRef.value.collapseSet(true);
    }
  }
};

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

watch(
  () => assetStore.selectedSchemaVariants.length,
  (newVal, oldVal) => {
    if (newVal > 1 && oldVal < 2 && leftResizablePanelRef.value) {
      leftResizablePanelRef.value.subpanelCollapseSet(true);
    } else if (newVal === 1 && oldVal > 1 && leftResizablePanelRef.value) {
      leftResizablePanelRef.value.subpanelCollapseSet(false);
    }
  },
);
</script>
