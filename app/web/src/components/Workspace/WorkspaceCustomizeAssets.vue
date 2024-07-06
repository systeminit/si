<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <component
    :is="ResizablePanel"
    ref="leftResizablePanelRef"
    :minSize="300"
    rememberSizeKey="func-picker"
    side="left"
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
      <AssetFuncListPanel :schemaVariantId="selectedVariantId" />
    </template>
  </component>

  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold relative"
  >
    <div class="absolute left-0 right-0 top-0 bottom-0">
      <FuncEditor
        v-if="selectedVariantId && selectedFuncId"
        :funcId="selectedFuncId"
      />
      <AssetEditor
        v-else-if="selectedVariantId"
        :schemaVariantId="selectedVariantId"
      />
      <WorkspaceCustomizeEmptyState
        v-else
        :instructions="
          assetStore.selectedSchemaVariants.length > 1
            ? 'You have selected multiple assets, use the right pane!'
            : undefined
        "
        :requestStatus="loadAssetsRequestStatus"
        loadingMessage="Loading assets..."
      />
    </div>
  </div>

  <component
    :is="ResizablePanel"
    ref="rightResizablePanelRef"
    :minSize="300"
    rememberSizeKey="func-details"
    side="right"
  >
    <div class="absolute w-full flex flex-col h-full">
      <AssetCard
        v-if="selectedVariantId"
        :assetId="selectedVariantId"
        titleCard
      />
      <template v-if="selectedVariantId">
        <FuncDetails
          v-if="
            selectedFuncId && assetStore.selectedSchemaVariant?.schemaVariantId
          "
          :funcId="selectedFuncId"
          :schemaVariantId="assetStore.selectedSchemaVariant?.schemaVariantId"
          allowTestPanel
          singleModelScreen
          @expand-panel="rightResizablePanelRef?.maximize()"
        />
        <!-- the key here is to force remounting so we get the proper asset
        request statuses -->
        <AssetDetailsPanel
          v-else
          :key="selectedVariantId"
          :schemaVariantId="selectedVariantId"
        />
      </template>
      <template v-else-if="assetStore.selectedSchemaVariants.length > 1">
        <div class="flex flex-col h-full w-full overflow-hidden">
          <ScrollArea>
            <template #top>
              <SidebarSubpanelTitle icon="multiselect" label="Multiple Assets">
                <DetailsPanelMenuIcon @click="open" />
              </SidebarSubpanelTitle>
              <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
            </template>

            <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
              {{ assetStore.selectedSchemaVariants.length }} assets selected:
            </div>
            <Stack class="p-xs" spacing="xs">
              <AssetCard
                v-for="assetId in assetStore.selectedSchemaVariants"
                :key="assetId"
                :assetId="assetId"
                :titleCard="false"
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
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";
import EmptyStateCard from "../EmptyStateCard.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";
import DetailsPanelMenuIcon from "../DetailsPanelMenuIcon.vue";
import AssetEditor from "../AssetEditor.vue";
import FuncEditor from "../FuncEditor/FuncEditor.vue";
import WorkspaceCustomizeEmptyState from "../WorkspaceCustomizeEmptyState.vue";

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const selectedVariantId = computed(() => assetStore.selectedVariantId);
const selectedFuncId = computed(() => funcStore.selectedFuncId);
const loadAssetsRequestStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT_LIST",
);

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
