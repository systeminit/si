<template>
  <div class="overflow-scroll">
    <template v-if="featureFlagsStore.ADMIN_PAGE">
      <div>
        <h2 class="pb-md font-bold">RUM Report</h2>
        <p class="pb-sm text-sm text-neutral-600 dark:text-neutral-400">
          Resources Under Management by workspace owner for the selected month
        </p>

        <Stack>
          <VormInput
            v-model="selectedMonth"
            label="Month"
            placeholder="YYYY-MM (e.g., 2025-10 or leave empty for current month)"
            @keydown.enter="loadRumReport()"
          />
          <VButton
            :requestStatus="rumReportReqStatus"
            iconRight="chevron--right"
            loadingText="Loading..."
            tone="action"
            variant="solid"
            @click="loadRumReport()"
          >
            Load Report
          </VButton>
        </Stack>

        <template v-if="rumReportReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="rumReportReqStatus.isError">
          <ErrorMessage :requestStatus="rumReportReqStatus" />
        </template>
        <template v-else-if="rumReportReqStatus.isSuccess">
          <div class="relative mt-4">
            <div class="text-lg font-bold mb-2">
              Total Entries: {{ rumReport.length }}
            </div>
            <table
              class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
            >
              <thead>
                <tr
                  class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                >
                  <th scope="col">Owner Name</th>
                  <th scope="col">Owner Email</th>
                  <th scope="col">Signup Date</th>
                  <th scope="col">Max RUM During Month</th>
                  <th scope="col">Owner ID</th>
                </tr>
              </thead>
              <tbody
                class="divide-y divide-neutral-300 dark:divide-neutral-700"
              >
                <tr
                  v-for="entry in rumReport"
                  :key="entry.id"
                  class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
                >
                  <td class="normal-case">
                    {{ entry.nickname }}
                  </td>
                  <td class="normal-case">
                    {{ entry.email }}
                  </td>
                  <td class="normal-case">
                    {{ entry.signupAt }}
                  </td>
                  <td class="normal-case font-mono">
                    {{ entry.maxRum }}
                  </td>
                  <td class="">
                    <div
                      class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate font-mono text-xs"
                    >
                      {{ entry.id }}
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
    </template>
    <template v-else> Feature not Enabled for account </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
} from "@si/vue-lib/design-system";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const selectedMonth = ref("");

const rumReportReqStatus = workspacesStore.getRequestStatus("GET_RUM_REPORT");
const rumReport = computed(() => workspacesStore.rumReport);

async function loadRumReport() {
  await workspacesStore.GET_RUM_REPORT(selectedMonth.value || undefined);
}

// Load report for current month on mount
if (authStore.userIsLoggedIn && featureFlagsStore.ADMIN_PAGE) {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  loadRumReport();
}
</script>
