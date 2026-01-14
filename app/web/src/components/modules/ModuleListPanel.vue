<template>
  <div class="flex flex-col overflow-hidden h-full relative">
    <SidebarSubpanelTitle icon="component">
      <template #label>Modules</template>
      <div>
        <IconButton
          :requestStatus="installModuleFromFileReqStatus"
          icon="cloud-upload"
          loadingIcon="loader"
          size="sm"
          tooltip="Install from file"
          tooltipPlacement="top"
          @click="openFilePicker"
        />
      </div>
    </SidebarSubpanelTitle>
    <input ref="specFileSelectorRef" class="hidden" type="file" @change="handleFileChange" />

    <SiSearch v-model="textSearch" class="flex-none" placeholder="search modules" />

    <ModuleList
      :loading="loadBuiltinsReqStatus"
      :modules="filteredBuiltins"
      :textSearch="textSearch"
      label="Builtin List"
      loadingMessage="Loading builtins..."
      noModulesMessage="No builtins found"
    />

    <ModuleList
      :loading="searchRemoteModulesReqStatus"
      :modules="filteredRemoteList"
      :textSearch="textSearch"
      label="Remote"
      loadingMessage="Loading remote modules..."
    />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, ref } from "vue";
import { IconButton, SiSearch } from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";
import ModuleList from "./ModuleList.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";

const moduleStore = useModuleStore();
const loadBuiltinsReqStatus = moduleStore.getRequestStatus("LIST_BUILTINS");
const searchRemoteModulesReqStatus = moduleStore.getRequestStatus("GET_REMOTE_MODULES_LIST");

const specFileSelectorRef = ref();
const installModuleFromFileReqStatus = moduleStore.getRequestStatus("INSTALL_MODULE_FROM_FILE");
const openFilePicker = () => {
  specFileSelectorRef.value.click();
};

const handleFileChange = (e: Event) => {
  if (!(e.target instanceof HTMLInputElement)) {
    return;
  }
  if (e.target.files?.length !== 1) {
    return;
  }

  const file = e.target.files[0];
  if (file) {
    moduleStore.INSTALL_MODULE_FROM_FILE(file);
  }
};

const textSearch = ref("");

const filteredBuiltins = computed(() => {
  if (!textSearch.value) return moduleStore.builtins;

  return _.filter(moduleStore.builtins, (m) => m.name.toLowerCase().includes(textSearch.value.toLowerCase()));
});

const filteredRemoteList = computed(() => {
  return _.filter(moduleStore.remoteModuleList, (m) => m.name.toLowerCase().includes(textSearch.value.toLowerCase()));
});

onMounted(() => {
  moduleStore.GET_REMOTE_MODULES_LIST({ su: true });
  moduleStore.LIST_BUILTINS();
});
</script>
