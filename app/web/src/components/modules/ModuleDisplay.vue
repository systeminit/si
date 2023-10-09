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
      <ErrorMessage :requestStatus="loadLocalModulesReqStatus" />
    </div>
    <div v-else-if="loadLocalModulesReqStatus.isSuccess">
      <template v-if="localSummary">
        <div
          class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none"
        >
          <div>
            <span class="font-bold">Hash:</span>
            {{ localSummary.hash }}
          </div>
          <div>
            <span class="font-bold">Created At: </span>
            <Timestamp
              :date="remoteDetails?.createdAt || localDetails?.createdAt"
              size="long"
            />
          </div>
          <div>
            <span class="font-bold">Created By: </span
            >{{ remoteDetails?.ownerDisplayName || localDetails?.createdBy }}
          </div>
        </div>

        <ErrorMessage
          v-if="!remoteSummary && !builtinSummary"
          tone="warning"
          class="mb-sm"
        >
          Module only exists locally
        </ErrorMessage>

        <div
          class="border dark:border-neutral-600 rounded flex flex-col gap-sm overflow-auto"
        >
          <template v-if="interactWithBuiltin">
            <ErrorMessage :requestStatus="rejectReqStatus" />
            <VButton
              :requestStatus="rejectReqStatus"
              :disabled="!builtinSummary"
              @click="rejectModuleSpecHandler"
            >
              Reject this builtin
            </VButton>

            <ErrorMessage :requestStatus="promoteToBuiltinReqStatus" />
            <VButton
              :requestStatus="promoteToBuiltinReqStatus"
              :disabled="builtinSummary && builtinSummary.isBuiltin"
              @click="promoteToBuiltinSpecHandler"
            >
              Promote this module to be a builtin
            </VButton>
          </template>

          <div
            class="px-sm py-xs border-b dark:border-neutral-600 font-bold flex-none"
          >
            Functions
          </div>

          <ul class="p-sm overflow-y-auto">
            <li
              v-for="func in remoteDetails?.metadata?.funcs ||
              localDetails?.funcs"
              :key="func.name"
              class="flex flex-col"
            >
              <div class="flex flex-row items-center">
                <div>
                  <i>{{ func.name }}</i>
                  <span v-if="func.displayName">: {{ func.displayName }}</span>
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
              v-for="sv in remoteDetails?.metadata?.schemas ||
              localDetails?.schemas"
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

      <!-- this else means the module does not exist locally -->
      <template v-else>
        <!-- deal with showing an error message if name is totally bogus -->
        <template v-if="loadRemoteModulesReqStatus.isSuccess && !remoteSummary">
          <ErrorMessage>
            Could not find module with hash {{ moduleSlug }}
          </ErrorMessage>
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

            <ErrorMessage :requestStatus="installReqStatus" />
            <VButton
              :requestStatus="installReqStatus"
              @click="installButtonHandler"
            >
              Install this module
            </VButton>

            <ErrorMessage :requestStatus="remoteModuleSpecStatus" />
            <VButton
              :requestStatus="remoteModuleSpecStatus"
              @click="viewModuleSpecHandler"
            >
              View functions from this module
            </VButton>

            <ErrorMessage :requestStatus="rejectReqStatus" />
            <VButton
              :requestStatus="rejectReqStatus"
              @click="rejectModuleSpecHandler"
            >
              Reject this module
            </VButton>

            <ErrorMessage :requestStatus="promoteToBuiltinReqStatus" />
            <VButton
              :requestStatus="promoteToBuiltinReqStatus"
              @click="promoteToBuiltinSpecHandler"
            >
              Promote this module to be a builtin
            </VButton>

            <div v-if="remoteSpec && remoteSpec.funcs.length > 0">
              <ul>
                <li
                  v-for="func in remoteSpec.funcs"
                  :key="func.uniqueId"
                  class="mt-5"
                >
                  <b>{{ func.name }}</b>
                  <CodeViewer
                    v-if="func.codeBase64"
                    :code="decodeb64(func.codeBase64)"
                    codeLanguage="javascript"
                  />
                  <p v-else>(builtin, or no code)</p>
                </li>
              </ul>
            </div>
          </Stack>
        </template>
      </template>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, watch } from "vue";
import {
  Icon,
  Timestamp,
  ErrorMessage,
  VormInput,
  Inline,
  Stack,
  VButton,
} from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";
import CodeViewer from "../CodeViewer.vue";

const moduleStore = useModuleStore();
const loadLocalModulesReqStatus =
  moduleStore.getRequestStatus("LOAD_LOCAL_MODULES");
const loadRemoteModulesReqStatus = moduleStore.getRequestStatus(
  "SEARCH_REMOTE_MODULES",
);

const _localDetailsReq = moduleStore.getRequestStatus(
  "GET_LOCAL_MODULE_DETAILS",
);
const _remoteDetailsReq = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULE_DETAILS",
);
const installReqStatus = moduleStore.getRequestStatus("INSTALL_REMOTE_MODULE");
const remoteModuleSpecStatus = moduleStore.getRequestStatus(
  "GET_REMOTE_MODULE_SPEC",
);

const rejectReqStatus = moduleStore.getRequestStatus("REJECT_REMOTE_MODULE");
const promoteToBuiltinReqStatus =
  moduleStore.getRequestStatus("PROMOTE_TO_BUILTIN");

const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);

const localSummary = computed(() => moduleStore.selectedModuleLocalSummary);
const localDetails = computed(() => moduleStore.selectedModuleLocalDetails);
const remoteSummary = computed(() => moduleStore.selectedModuleRemoteSummary);
const remoteDetails = computed(
  () =>
    moduleStore.selectedModuleRemoteDetails ||
    moduleStore.selectedBuiltinModuleDetails,
);
const builtinSummary = computed(() => moduleStore.selectedBuiltinModuleSummary);
const interactWithBuiltin = computed(
  () => builtinSummary.value || !localSummary.value?.isBuiltin,
);
const remoteSpec = computed(() =>
  remoteSummary.value?.id
    ? moduleStore.remoteModuleSpecsById[remoteSummary.value?.id]
    : undefined,
);

// since the URL is based on the name, but we need the hash to fetch the module details
// we have to wait until we have the local info loaded
watch(
  localSummary,
  () => {
    if (localSummary.value) {
      moduleStore.GET_LOCAL_MODULE_DETAILS(localSummary.value.hash);
    }
  },
  { immediate: true },
);

watch(
  remoteSummary,
  () => {
    if (remoteSummary.value) {
      moduleStore.GET_REMOTE_MODULE_DETAILS(remoteSummary.value.id);
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

async function rejectModuleSpecHandler() {
  if (!remoteSummary.value) return;
  await moduleStore.REJECT_REMOTE_MODULE(remoteSummary.value?.id);
}

async function promoteToBuiltinSpecHandler() {
  if (!remoteSummary.value) return;
  await moduleStore.PROMOTE_TO_BUILTIN(remoteSummary.value?.id);
}

async function viewModuleSpecHandler() {
  if (!remoteSummary.value) return;
  await moduleStore.GET_REMOTE_MODULE_SPEC(remoteSummary.value?.id);
}

const decodeb64 = (input: string) => atob(input);
</script>
