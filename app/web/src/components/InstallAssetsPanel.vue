<template>
  <ScrollArea>
    <RequestStatusMessage
      v-if="loadModulesReqStatus.isPending"
      :requestStatus="loadModulesReqStatus"
      loadingMessage="Loading modules..."
    />
    <template #top>
      <SidebarSubpanelTitle icon="component">
        <template #label>
          <div class="flex flex-row gap-xs">
            <div>Install New Assets</div>
          </div>
        </template>
      </SidebarSubpanelTitle>
      <SiSearch
        ref="searchRef"
        placeholder="search assets"
        @search="onSearch"
      />
    </template>
    <ul
      v-if="assetStore.installableModules.length > 0"
      :class="
        clsx(
          'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
          'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
        )
      "
    >
      <li
        v-for="module in filteredModules"
        :key="module.id"
        :class="
          clsx(
            'text-xs w-full p-2xs truncate flex flex-row items-center gap-1 hover:text-action-500 dark:hover:text-action-300 cursor-pointer',
            selectedModule &&
              module.id === selectedModule.id &&
              themeClasses('bg-action-100', 'bg-action-900'),
          )
        "
        @click="() => selectModule(module)"
      >
        <div class="truncate">
          {{ module.name }}
        </div>
      </li>
    </ul>
    <EmptyStateCard
      v-else
      iconName="no-assets"
      primaryText="No Installable Assets"
      secondaryText="Check back later when more assets are contributed."
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { ref, computed } from "vue";
import {
  ScrollArea,
  RequestStatusMessage,
  themeClasses,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { useAssetStore } from "@/store/asset.store";
import { Module } from "@/api/sdf/dal/schema";
import router from "@/router";
import EmptyStateCard from "./EmptyStateCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";

const assetStore = useAssetStore();

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const filteredModules = computed(() =>
  assetStore.installableModules.filter((m) =>
    m.name.toLocaleLowerCase().includes(searchString.value),
  ),
);

const loadModulesReqStatus = assetStore.getRequestStatus("LOAD_MODULES");

const selectedModule = ref<Module | undefined>();

const selectModule = (module: Module) => {
  selectedModule.value = module;
  const newQueryObj = {
    ...{ m: module.id },
  };
  router.replace({
    query: newQueryObj,
  });
};

defineExpose({ selectedModule });
</script>
