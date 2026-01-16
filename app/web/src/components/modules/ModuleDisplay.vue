<template>
  <div v-if="moduleSlug && loadRemoteModulesReqStatus.isSuccess" class="inset-0 p-sm absolute overflow-auto">
    <div v-if="builtinSummary || remoteSummary" class="flex flex-row items-center gap-xs flex-none">
      <Icon name="component" />
      <div class="text-3xl font-bold truncate">
        {{ builtinSummary?.name || remoteSummary?.name || moduleStore.urlSelectedModuleSlug }}
      </div>
    </div>

    <!-- A builtin is selected -->
    <template v-if="builtinSummary">
      <div class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none">
        <div>
          <span class="font-bold">Hash:</span>
          {{ builtinSummary.hash }}
        </div>
        <div>
          <span class="font-bold">Created At: </span>
          <Timestamp :date="remoteDetails?.createdAt || builtinSummary?.createdAt" size="long" />
        </div>
        <div>
          <span class="font-bold">Created By: </span>{{ remoteDetails?.ownerDisplayName || builtinDetails?.createdAt }}
        </div>
      </div>

      <ErrorMessage v-if="!remoteSummary && !builtinSummary" tone="warning" class="mb-sm">
        Module doesn't exist
      </ErrorMessage>

      <div class="border dark:border-neutral-600 rounded flex flex-col gap-sm overflow-auto">
        <template v-if="builtinSummary">
          <ErrorMessage :requestStatus="rejectReqStatus" />
          <VButton :requestStatus="rejectReqStatus" :disabled="!builtinSummary" @click="rejectBuiltinSpecHandler">
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

        <div class="px-sm py-xs border-b dark:border-neutral-600 font-bold flex-none">Functions</div>

        <ul class="p-sm overflow-y-auto">
          <li v-for="func in remoteDetails?.metadata?.funcs" :key="func.name" class="flex flex-col">
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

        <div class="px-sm py-xs border-b border-t my-xs dark:border-neutral-600 font-bold flex-none">
          Schema Variants
        </div>

        <ul class="p-sm overflow-y-auto">
          <li v-for="sv in remoteDetails?.metadata?.schemas" :key="sv" class="flex flex-col">
            <div class="flex flex-row items-center">
              <div>{{ sv }}</div>
            </div>
            <div class="pl-lg pb-sm">other info goes here</div>
          </li>
        </ul>
      </div>
    </template>

    <!-- A remote module is selected -->
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

        <ErrorMessage tone="warning"> Module is not currently installed locally </ErrorMessage>

        <ErrorMessage :requestStatus="installReqStatus" />
        <ErrorMessage :message="installError" />
        <VButton :requestStatus="installReqStatus" :loading="installReqStatus.isPending" @click="installButtonHandler">
          Install this module
        </VButton>

        <ErrorMessage :requestStatus="remoteModuleSpecStatus" />
        <VButton :requestStatus="remoteModuleSpecStatus" @click="viewModuleSpecHandler">
          View functions from this module
        </VButton>

        <ErrorMessage :requestStatus="rejectReqStatus" />
        <VButton :requestStatus="rejectReqStatus" @click="rejectModuleSpecHandler"> Reject this module </VButton>

        <ErrorMessage :requestStatus="promoteToBuiltinReqStatus" />
        <VButton :requestStatus="promoteToBuiltinReqStatus" @click="promoteToBuiltinSpecHandler">
          Promote this module to be a builtin
        </VButton>

        <div v-if="remoteSpec && remoteSpec.funcs.length > 0">
          <ul>
            <li v-for="func in remoteSpec.funcs" :key="func.uniqueId" class="mt-5">
              <b>{{ func.name }}</b>
              <CodeViewer
                v-if="func.data.codeBase64"
                :code="decodeb64(func.data.codeBase64)"
                codeLanguage="javascript"
              />
              <p v-else>(builtin, or no code)</p>
            </li>
          </ul>
        </div>
      </Stack>
    </template>

    <!-- deal with showing an error message if name is totally bogus -->
    <template v-else>
      <ErrorMessage>
        Could not find module with hash
        <span class="italic font-bold">"{{ moduleSlug }}"</span>
      </ErrorMessage>
    </template>
  </div>
  <WorkspaceCustomizeEmptyState v-else :requestStatus="loadBuiltsReqStatus" loadingMessage="Loading builtins..." />
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, watch, ref } from "vue";
import { Icon, Timestamp, ErrorMessage, VormInput, Inline, Stack, VButton } from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";
import CodeViewer from "../CodeViewer.vue";
import WorkspaceCustomizeEmptyState from "../WorkspaceCustomizeEmptyState.vue";

const moduleStore = useModuleStore();
const loadBuiltsReqStatus = moduleStore.getRequestStatus("LIST_BUILTINS");
const loadRemoteModulesReqStatus = moduleStore.getRequestStatus("GET_REMOTE_MODULES_LIST");

const remoteModuleSpecStatus = moduleStore.getRequestStatus("GET_REMOTE_MODULE_SPEC");

const rejectReqStatus = moduleStore.getRequestStatus("REJECT_REMOTE_MODULE");
const promoteToBuiltinReqStatus = moduleStore.getRequestStatus("PROMOTE_TO_BUILTIN");

const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);

const remoteSummary = computed(() => moduleStore.selectedModuleRemoteSummary);
const remoteDetails = computed(() => moduleStore.selectedModuleRemoteDetails);
const builtinSummary = computed(() => moduleStore.selectedBuiltinModuleSummary);
const builtinDetails = computed(() => moduleStore.selectedBuiltinModuleDetails);
const remoteSpec = computed(() =>
  remoteSummary.value?.id ? moduleStore.remoteModuleSpecsById[remoteSummary.value?.id] : undefined,
);

const remoteSummaryId = computed(() => remoteSummary.value?.id);
const installReqStatus = moduleStore.getRequestStatus("INSTALL_REMOTE_MODULE", remoteSummaryId);

watch(
  builtinSummary,
  () => {
    if (builtinSummary.value) {
      moduleStore.GET_REMOTE_MODULE_DETAILS(builtinSummary.value.id);
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

const installError = ref<string | undefined>();
async function installButtonHandler() {
  installError.value = undefined;
  if (!remoteSummary.value) return;
  const resp = await moduleStore.INSTALL_REMOTE_MODULE([remoteSummary.value?.id]);
  if (!resp.result.success) {
    installError.value = resp.result.err.message;
  }
}

async function rejectModuleSpecHandler() {
  if (!remoteSummary.value) return;
  await moduleStore.REJECT_REMOTE_MODULE(remoteSummary.value?.id);
}

async function rejectBuiltinSpecHandler() {
  if (!builtinSummary.value) return;
  await moduleStore.REJECT_REMOTE_MODULE(builtinSummary.value?.id);
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
