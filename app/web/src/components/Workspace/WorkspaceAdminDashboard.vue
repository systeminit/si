<template>
  <div
    class="w-full h-full flex flex-col gap-xs p-lg items-center relative overflow-scroll dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <span class="font-bold text-3xl">Admin Dashboard</span>
    <span class="text-xs">commit hash: {{ commitHash }}</span>
    <span class="text-xs">shared worker hash: {{ sharedWorkerHash }}</span>
    <div class="flex flex-row gap-sm w-full">
      <Stack spacing="md" class="flex-none">
        <Stack class="max-w-xl">
          <h2 class="font-bold text-lg">UPDATE MODULE CACHE</h2>
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="adminStore.updatingModuleCacheOperationRunning"
              class="flex-grow"
              icon="plus-circle"
              label="Update module cache"
              loadingText="Updating module cache"
              :loading="adminStore.updatingModuleCacheOperationRunning"
              tone="success"
              @click="updateModuleCache"
            />
          </div>
        </Stack>
        <Stack class="max-w-xl">
          <h2 class="font-bold text-lg">CLEAR INNIT PARAMETER CACHE</h2>
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="adminStore.clearingInnitCacheOperationRunning"
              class="flex-grow"
              icon="plus-circle"
              label="Clear Innit parameter cache"
              loadingText="Clearing Innit parameter cache"
              :loading="adminStore.clearingInnitCacheOperationRunning"
              tone="success"
              @click="clearInnitCache"
            />
          </div>
        </Stack>
        <Stack class="max-w-xl">
          <h2 class="font-bold text-lg">KILL FUNCTION EXECUTION</h2>
          <VormInput
            v-model="funcRunId"
            label="FuncRunId for function execution"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="!funcRunId"
              :requestStatus="killExecutionReqStatus"
              class="flex-grow"
              icon="plus-circle"
              label="Kill function execution"
              loadingText="Killing function execution"
              tone="success"
              @click="killExecution"
            />
          </div>
        </Stack>
        <Stack class="text-xs font-bold" spacing="none">
          <div class="text-lg pb-2xs">Feature Flags:</div>
          <div
            v-for="flag in featureFlags"
            :key="flag.name"
            :class="
              clsx(
                'flex flex-row p-2xs',
                themeClasses('', 'odd:bg-neutral-600'),
              )
            "
          >
            <div class="flex-1">{{ flag.name }}:</div>
            <div
              :class="
                clsx(
                  flag.value ? 'text-success-500' : 'text-destructive-500',
                  'uppercase flex-none',
                )
              "
            >
              {{ flag.value }}
            </div>
          </div>
        </Stack>
      </Stack>
      <WorkspaceAdmin class="flex-1 min-w-0 overflow-hidden" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeMount, ref } from "vue";
import {
  Stack,
  VormInput,
  VButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import clsx from "clsx";
import { useAdminStore } from "@/store/admin.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import WorkspaceAdmin from "@/components/AdminDashboard/WorkspaceAdmin.vue";

const adminStore = useAdminStore();
const featureFlagsStore = useFeatureFlagsStore();

const router = useRouter();
onBeforeMount(async () => {
  if (!featureFlagsStore.ADMIN_PANEL_ACCESS) {
    await router.push({ name: "workspace-single" });
  }
});

const updateModuleCache = async () => {
  await adminStore.UPDATE_MODULE_CACHE();
};

const clearInnitCache = async () => {
  await adminStore.CLEAR_INNIT_CACHE();
};

const killExecutionReqStatus = adminStore.getRequestStatus("KILL_EXECUTION");

const killExecution = () => {
  if (funcRunId.value) {
    adminStore.KILL_EXECUTION(funcRunId.value);
  }
};

const funcRunId = ref<string | null>(null);

const commitHash = __COMMIT_HASH__;
const sharedWorkerHash = __SHARED_WORKER_HASH__;

const featureFlags = computed(() => {
  return featureFlagsStore.allFeatureFlags;
});
</script>
