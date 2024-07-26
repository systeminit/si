<template>
  <div class="flex flex-col overflow-hidden h-full relative">
    <SidebarSubpanelTitle icon="component">
      <template #label>Modules</template>
    </SidebarSubpanelTitle>

    <SiSearch
      v-model="textSearch"
      class="flex-none"
      placeholder="search modules"
    />

    <ModuleList
      label="Local / Installed"
      :modules="filteredLocalModules"
      :loading="loadLocalModulesReqStatus"
      loadingMessage="Loading local modules..."
      :textSearch="textSearch"
      noModulesMessage="No modules installed"
    />

    <ModuleList
      label="Remote"
      :modules="filteredRemoteList"
      :loading="searchRemoteModulesReqStatus"
      loadingMessage="Loading remote modules..."
      :textSearch="textSearch"
    />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, ref } from "vue";
import SiSearch from "@/components/SiSearch.vue";
import { useModuleStore } from "@/store/module.store";
import ModuleList from "./ModuleList.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";

const moduleStore = useModuleStore();
const loadLocalModulesReqStatus =
  moduleStore.getRequestStatus("LOAD_LOCAL_MODULES");
const searchRemoteModulesReqStatus = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULES_LIST",
);

const textSearch = ref("");

const filteredLocalModules = computed(() => {
  if (!textSearch.value) return moduleStore.localModules;

  return _.filter(moduleStore.localModules, (m) =>
    m.name.toLowerCase().includes(textSearch.value.toLowerCase()),
  );
});

const filteredRemoteList = computed(() => {
  return _.filter(moduleStore.remoteModuleList, (m) =>
    m.name.toLowerCase().includes(textSearch.value.toLowerCase()),
  );
});

onMounted(() => {
  moduleStore.GET_REMOTE_MODULES_LIST({ su: true });
});
</script>
