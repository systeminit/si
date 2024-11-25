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
      label="Builtin List"
      :modules="filteredBuiltins"
      :loading="loadBuiltinsReqStatus"
      loadingMessage="Loading builtins..."
      :textSearch="textSearch"
      noModulesMessage="No builtins found"
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
import { SiSearch } from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";
import ModuleList from "./ModuleList.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";

const moduleStore = useModuleStore();
const loadBuiltinsReqStatus = moduleStore.getRequestStatus("LIST_BUILTINS");
const searchRemoteModulesReqStatus = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULES_LIST",
);

const textSearch = ref("");

const filteredBuiltins = computed(() => {
  if (!textSearch.value) return moduleStore.builtins;

  return _.filter(moduleStore.builtins, (m) =>
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
