<template>
  <div class="flex flex-col overflow-hidden h-full relative">
    <SiSearch
      v-model="textSearch"
      class="flex-none"
      autoSearch
      placeholder="search modules"
      @search="triggerSearch"
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
      :modules="moduleStore.remoteModuleSearchResults"
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

const moduleStore = useModuleStore();
const loadLocalModulesReqStatus =
  moduleStore.getRequestStatus("LOAD_LOCAL_MODULES");
const searchRemoteModulesReqStatus = moduleStore.getRequestStatus(
  "SEARCH_REMOTE_MODULES",
);

const filteredLocalModules = computed(() => {
  if (!textSearch.value) return moduleStore.localModules;

  return _.filter(moduleStore.localModules, (m) =>
    m.name.toLowerCase().includes(textSearch.value.toLowerCase()),
  );
});

const textSearch = ref("");

async function triggerSearch() {
  await moduleStore.SEARCH_REMOTE_MODULES({ name: textSearch.value, su: true });
}

onMounted(triggerSearch);
</script>
