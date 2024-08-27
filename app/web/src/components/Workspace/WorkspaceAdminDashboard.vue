<template>
  <div
    class="w-full h-full flex flex-col items-center relative overflow-hidden dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <Stack spacing="lg">
      <span class="flex flex-row mt-10 font-bold text-3xl"
        >Admin Dashboard</span
      >
      <Stack>
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
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { onBeforeMount, ref } from "vue";
import { Stack, VormInput, VButton } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useAdminStore } from "@/store/admin.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const adminStore = useAdminStore();
const featureFlagStore = useFeatureFlagsStore();

const router = useRouter();
onBeforeMount(async () => {
  if (!featureFlagStore.ADMIN_PANEL_ACCESS) {
    await router.push({ name: "workspace-single" });
  }
});

const killExecutionReqStatus = adminStore.getRequestStatus("KILL_EXECUTION");

const funcRunId = ref<string | null>(null);

const killExecution = () => {
  if (funcRunId.value) {
    adminStore.KILL_EXECUTION(funcRunId.value);
  }
};
</script>
