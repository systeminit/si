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

          <ErrorMessage v-if="!remoteSummary" tone="warning" class="mb-sm">
            Module only exists locally
          </ErrorMessage>

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

      <!-- this else means the module does not exist locally -->
      <template v-else>
        <!-- deal with showing an error message if name is totally bogus -->
        <template v-if="loadRemoteModulesReqStatus.isSuccess && !remoteSummary">
          <ErrorMessage
            >Could not find any module with name {{ moduleSlug }}</ErrorMessage
          >
        </template>
        <template v-else-if="remoteSummary">
          <Stack class="mt-md">
            <Inline spacing="lg">
              <VormInput type="container" label="Owner/Creator">
                {{ remoteSummary.ownerDisplayName }}
              </VormInput>
              <VormInput type="container" label="Created Date">
                <Timestamp :date="remoteSummary.createdAt" />
              </VormInput>
            </Inline>
            <p class="text-lg">{{ remoteSummary.description }}</p>

            <ErrorMessage tone="warning">
              Module is not currently installed locally
            </ErrorMessage>

            <ErrorMessage :request-status="installReqStatus" />
            <VButton
              :request-status="installReqStatus"
              @click="installButtonHandler"
              >Install this module</VButton
            >
          </Stack>
        </template>
      </template>
    </div>

    <!-- <div
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
    </div> -->
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, watch } from "vue";
import {
  Icon,
  RequestStatusMessage,
  Timestamp,
  ErrorMessage,
  VormInput,
  Tiles,
  Inline,
  Stack,
  VButton,
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
const installReqStatus = moduleStore.getRequestStatus("INSTALL_REMOTE_MODULE");

const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);

const localSummary = computed(() => moduleStore.selectedModuleLocalSummary);
const localDetails = computed(() => moduleStore.selectedModuleLocalDetails);
const remoteSummary = computed(
  () =>
    moduleStore.remoteModuleSummaryByName[moduleStore.urlSelectedModuleSlug!],
);
const remoteDetails = computed(() => moduleStore.selectedModuleRemoteDetails);

// since the URL is based on the name, but we need the hash to fetch the module details
// we have to wait until we have the local info loaded
watch(
  localSummary,
  () => {
    if (localSummary.value) {
      moduleStore.GET_LOCAL_MODULE_DETAILS(localSummary.value?.hash);
    }
  },
  { immediate: true },
);

onBeforeMount(() => {
  if (!moduleStore.urlSelectedModuleSlug) return; // can't happen, but makes TS happy
});

async function installButtonHandler() {
  if (!remoteSummary.value) return;
  await moduleStore.INSTALL_REMOTE_MODULE(remoteSummary.value?.id);
}
</script>
