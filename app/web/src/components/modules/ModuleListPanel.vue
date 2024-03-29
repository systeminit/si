<template>
  <ScrollArea>
    <template #top>
      <div
        class="p-xs border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      ></div>
      <SiSearch
        v-model="textSearch"
        autoSearch
        placeholder="search modules"
        @search="triggerSearch"
      />
    </template>

    <Collapsible label="Local / Installed" defaultOpen>
      <ErrorMessage :requestStatus="loadLocalModulesReqStatus" />
      <template v-if="loadLocalModulesReqStatus.isPending">
        Loading local modules...
      </template>
      <template v-else-if="loadLocalModulesReqStatus.isSuccess">
        <div
          v-if="!filteredLocalModules.length"
          class="p-sm italic text-center text-xs"
        >
          {{
            textSearch
              ? "No modules match the search criteria"
              : "No modules installed"
          }}
          .
        </div>

        <ModuleListItem
          v-for="p in filteredLocalModules"
          :key="p.hash"
          :moduleSlug="p.hash"
        />
      </template>
    </Collapsible>

    <Collapsible label="Remote" defaultOpen>
      <ErrorMessage :requestStatus="searchRemoteModulesReqStatus" />
      <template v-if="searchRemoteModulesReqStatus.isPending">
        Loading remote modules...
      </template>
      <template v-else-if="searchRemoteModulesReqStatus.isSuccess">
        <div
          v-if="!moduleStore.remoteModuleSearchResults.length"
          class="p-sm italic text-center text-xs"
        >
          No modules match your search
        </div>

        <ModuleListItem
          v-for="p in moduleStore.remoteModuleSearchResults"
          :key="p.id"
          :moduleSlug="p.hash"
        />
      </template>
    </Collapsible>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, ref } from "vue";
import {
  Collapsible,
  ErrorMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { useModuleStore } from "@/store/module.store";
import ModuleListItem from "./ModuleListItem.vue";

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
