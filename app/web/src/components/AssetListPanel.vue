<template>
  <ScrollArea>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending && assetStore.variantList.length < 1"
      :requestStatus="loadAssetsReqStatus"
      loadingMessage="Loading Assets..."
    />
    <template #top>
      <SidebarSubpanelTitle icon="component">
        <template #label>
          <div class="flex flex-row gap-xs">
            <div>Assets</div>
            <PillCounter :count="assetList.length" />
          </div>
        </template>
        <div class="flex flex-row gap-xs items-center">
          <IconButton
            :requestStatus="syncModulesReqStatus"
            class="hover:scale-125"
            icon="refresh"
            loadingIcon="loader"
            loadingTooltip="Checking For Asset Updates And For New Assets..."
            size="sm"
            tooltip="Check For Asset Updates Or Install New Assets"
            tooltipPlacement="top"
            variant="simple"
            @click="syncModules"
          />
          <IconButton
            :requestStatus="createAssetReqStatus"
            class="hover:scale-125"
            icon="plus"
            loadingIcon="loader"
            loadingTooltip="Creating Asset..."
            size="sm"
            tooltip="New Asset"
            tooltipPlacement="top"
            variant="simple"
            @click="() => newAssetModalRef?.modal?.open()"
          />
          <IconButton
            v-if="canUpdate"
            class="hover:scale-125"
            icon="code-deployed"
            size="sm"
            tooltip="Update All"
            tooltipPlacement="top"
            variant="simple"
            @click="updateAllAssets"
          />
        </div>
        <AssetNameModal
          ref="newAssetModalRef"
          :loading="createAssetReqStatus.isPending"
          buttonLabel="Create Asset"
          title="New Asset"
          @submit="(name) => newAsset(name)"
        />
      </SidebarSubpanelTitle>
      <SiSearch
        ref="searchRef"
        :filters="searchFiltersWithCounts"
        placeholder="search assets"
        @search="onSearch"
      />
      <!-- <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-xs border-b dark:border-neutral-600"
      >
        Select an asset to view or edit it.
      </div> -->
    </template>
    <template v-if="assetStore.variantList.length > 0">
      <TreeNode
        v-for="category in Object.keys(categorizedAssets).sort((a, b) =>
          a.localeCompare(b),
        )"
        :key="category"
        :color="categoryColor(category)"
        :label="category"
        :primaryIcon="getAssetIcon(category)"
        alwaysShowArrow
        clickLabelToToggle
        enableDefaultHoverClasses
        enableGroupToggle
        indentationSize="none"
      >
        <template #icons>
          <PillCounter
            :count="categorizedAssets[category]?.length || 0"
            showHoverInsideTreeNode
          />
        </template>
        <AssetListItem
          v-for="asset in categorizedAssets[category]?.sort((a, b) =>
            (a.displayName || a.schemaName)?.localeCompare(
              b.displayName || b.schemaName,
            ),
          )"
          :key="asset.schemaVariantId"
          :a="asset"
          :c="categorizedAssets[category]"
        />
      </TreeNode>
    </template>
    <Modal
      ref="contributeAssetSuccessModalRef"
      size="sm"
      title="Contribution sent"
    >
      <p>
        Thanks for contributing! We will review your contribution, and reach out
        via email or on our
        <a class="text-action-500" href="https://discord.com/invite/system-init"
          >Discord Server</a
        >
        if you have any questions.
      </p>
    </Modal>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import {
  ScrollArea,
  Modal,
  RequestStatusMessage,
  TreeNode,
  PillCounter,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import SiSearch, { Filter } from "@/components/SiSearch.vue";
import { useAssetStore } from "@/store/asset.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { getAssetIcon } from "@/store/components.store";
import { useModuleStore } from "@/store/module.store";
import AssetNameModal from "./AssetNameModal.vue";
import AssetListItem from "./AssetListItem.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import IconButton from "./IconButton.vue";

const assetStore = useAssetStore();
const moduleStore = useModuleStore();
const router = useRouter();

const { variantList: assetList } = storeToRefs(assetStore);

const createAssetReqStatus = assetStore.getRequestStatus("CREATE_VARIANT");
const loadAssetsReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT_LIST",
);
const syncModulesReqStatus = moduleStore.getRequestStatus("SYNC");

const contributeAssetSuccessModalRef = ref<InstanceType<typeof Modal>>();
const newAssetModalRef = ref<InstanceType<typeof AssetNameModal>>();

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const canUpdate = computed(
  () => Object.keys(moduleStore.upgradeableModules).length !== 0,
);

const categorizedAssets = computed(() =>
  assetList.value
    .filter((asset) => {
      let include = true;

      if (
        searchRef.value?.filteringActive &&
        searchRef.value?.activeFilters.filter(Boolean).length > 0
      ) {
        const idxs = searchRef.value?.activeFilters.flatMap((bool, idx) =>
          bool ? idx : [],
        );
        include = false;
        idxs.forEach((idx) => {
          if (filters.value[idx]?.includes(asset)) {
            include = true;
          }
        });
      }

      if (include && searchString.value.length) {
        include = !!(
          asset.schemaName.toLocaleLowerCase().includes(searchString.value) ||
          asset.displayName?.toLocaleLowerCase().includes(searchString.value) ||
          asset.category?.toLocaleLowerCase().includes(searchString.value)
        );
      }

      return include;
    })
    .reduce((categorized, asset) => {
      let catList = categorized[asset.category];
      if (!catList) {
        catList = [];
      }
      catList.push(asset);
      categorized[asset.category] = catList;
      return categorized;
    }, {} as { [key: string]: SchemaVariant[] }),
);

const categoryColor = (category: string) => {
  const assets = categorizedAssets.value[category];

  if (assets && assets[0]) {
    return assets[0].color;
  }

  return "#000";
};

const syncModules = async () => moduleStore.SYNC();

const newAsset = async (newAssetName: string) => {
  const result = await assetStore.CREATE_VARIANT(newAssetName);
  if (result.result.success) {
    assetStore.setSchemaVariantSelection(result.result.data.schemaVariantId);
    newAssetModalRef.value?.modal?.close();
  } else if (result.result.statusCode === 409) {
    newAssetModalRef.value?.setError("That name is already in use");
  }
  newAssetModalRef.value?.reset();
};

const updateAllAssets = () => {
  Object.values(moduleStore.upgradeableModules).forEach((module) => {
    moduleStore.INSTALL_REMOTE_MODULE(module.id);
  });
  router.replace({
    name: "workspace-lab-assets",
  });
};

const filters = computed(() => [
  assetList.value.filter((a) => a.canContribute),
  assetList.value.filter(
    (a) => !!moduleStore.upgradeableModules[a.schemaVariantId],
  ),
  assetList.value.filter((a) => !a.isLocked),
]);

const searchFiltersWithCounts = computed(() => {
  const searchFilters: Array<Filter> = [
    {
      name: "Assets to Contribute",
      iconTone: "action",
      iconName: "cloud-upload",
      count: filters.value[0]?.length,
    },
    {
      name: "Updates Available",
      iconTone: "action",
      iconName: "code-deployed",
      count: filters.value[1]?.length,
    },
    {
      name: "Editing Assets",
      iconTone: "action",
      iconName: "sliders-vertical",
      count: filters.value[2]?.length,
    },
  ];
  return searchFilters;
});
</script>
