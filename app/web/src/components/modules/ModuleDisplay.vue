<template>
  <div>
    <div class="flex flex-row items-center gap-2 flex-none">
      <Icon name="component" />
      <div class="text-3xl font-bold truncate">
        {{
          localSummary?.name ||
          remoteSummary?.name ||
          moduleStore.urlSelectedModuleSlug
        }}
      </div>
    </div>

    <div
      v-if="
        loadLocalModulesReqStatus.isPending ||
        !loadLocalModulesReqStatus.isRequested
      "
    >
      loading local modules...
    </div>
    <div v-else-if="loadLocalModulesReqStatus.isError">
      <ErrorMessage :request-status="loadLocalModulesReqStatus" />
    </div>
    <div v-else-if="loadLocalModulesReqStatus.isSuccess">
      <template v-if="localSummary">
        installed locally
        <template v-if="localDetails">
          <div
            class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none"
          >
            <div>
              <span class="font-bold">Version:</span>
              {{ localDetails.version }}
            </div>
            <div>
              <span class="font-bold">Created At: </span>
              <Timestamp :date="localDetails.createdAt" size="long" />
            </div>
            <div>
              <span class="font-bold">Created By: </span
              >{{ localDetails.createdBy }}
            </div>
          </div>
          <div
            class="border dark:border-neutral-600 rounded flex flex-col gap-sm overflow-auto"
          >
            <div
              class="px-sm py-xs border-b dark:border-neutral-600 font-bold flex-none"
            >
              Functions
            </div>

            <ul class="p-sm overflow-y-auto">
              <li
                v-for="func in localDetails.funcs"
                :key="func.name"
                class="flex flex-col"
              >
                <div class="flex flex-row items-center">
                  <div>
                    <i>{{ func.name }}</i>
                    <span v-if="func.displayName"
                      >: {{ func.displayName }}</span
                    >
                  </div>
                </div>
                <div class="pl-lg pb-sm">
                  {{ func.description }}
                </div>
              </li>
            </ul>

            <div
              class="px-sm py-xs border-b border-t my-xs dark:border-neutral-600 font-bold flex-none"
            >
              Schema Variants
            </div>

            <ul class="p-sm overflow-y-auto">
              <li
                v-for="sv in localDetails.schemas"
                :key="sv"
                class="flex flex-col"
              >
                <div class="flex flex-row items-center">
                  <div>{{ sv }}</div>
                </div>
                <div class="pl-lg pb-sm">other info goes here</div>
              </li>
            </ul>
          </div>
        </template>
      </template>
      <template v-else> Module NOT installed locally </template>
    </div>

    <hr />

    <div
      v-if="
        loadRemoteModulesReqStatus.isPending ||
        !loadRemoteModulesReqStatus.isRequested
      "
    >
      loading remote modules...
    </div>
    <div v-else-if="loadRemoteModulesReqStatus.isError">
      <ErrorMessage :request-status="loadRemoteModulesReqStatus" />
    </div>
    <div v-else-if="loadRemoteModulesReqStatus.isSuccess">
      <template v-if="remoteSummary">Module exists on remote! </template>
      <template v-else> Module NOT on index</template>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, watch } from "vue";
import {
  Icon,
  RequestStatusMessage,
  Timestamp,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";

const moduleStore = useModuleStore();
const loadLocalModulesReqStatus =
  moduleStore.getRequestStatus("LOAD_LOCAL_MODULES");
const loadRemoteModulesReqStatus = moduleStore.getRequestStatus(
  "SEARCH_REMOTE_MODULES",
);

const localDetailsReq = moduleStore.getRequestStatus(
  "GET_LOCAL_MODULE_DETAILS",
);
const remoteDetailsReq = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULE_DETAILS",
);

const localSummary = computed(() => moduleStore.selectedModuleLocalSummary);
const localDetails = computed(() => moduleStore.selectedModuleLocalDetails);
const remoteSummary = computed(
  () =>
    moduleStore.remoteModuleSummaryByName[moduleStore.urlSelectedModuleSlug!],
);
const remoteDetails = computed(() => moduleStore.selectedModuleRemoteDetails);

onBeforeMount(() => {
  if (!moduleStore.urlSelectedModuleSlug) return; // can't happen, but makes TS happy
  moduleStore.GET_LOCAL_MODULE_DETAILS(moduleStore.urlSelectedModuleSlug);
});
</script>
