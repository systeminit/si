<template>
  <Stack class="p-10 w-full">
    <h1>Validate Snapshot</h1>
    <LoadStatus :requestStatus="requestStatus">
      <template #loading>
        <div class="text-neutral-500">Validating snapshot ...</div>
      </template>
      <template #error>
        <div class="text-red-500">
          Error validating snapshot: {{ requestStatus.errorMessage }}
        </div>
      </template>
      <template #success>
        <div class="flex flex-row gap-xs p-xs w-full text-xs font-bold">
          <div class="text-lg">
            Validation: {{ allIssues.length }} Issues Found
          </div>
          <div v-for="[issueType, issues] in issuesByType" :key="issueType">
            <div class="text-lg">{{ issueType }}: {{ issues.length }}</div>
            <div v-for="issue in issues" :key="issue.message">
              {{ issue.message }}
            </div>
          </div>
        </div>
      </template>
    </LoadStatus>
  </Stack>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { Stack, LoadStatus } from "@si/vue-lib/design-system";
import { useAdminStore } from "@/store/admin.store";

const adminStore = useAdminStore();
const requestStatus = adminStore.getRequestStatus("VALIDATE_SNAPSHOT");
const allIssues = computed(
  () => adminStore.validateSnapshotResponse?.issues ?? [],
);
const issuesByType = computed(() =>
  _.sortBy(
    Object.entries(
      _.groupBy(
        adminStore.validateSnapshotResponse?.issues ?? [],
        (issue) => issue.type,
      ),
    ),
    (migrations) => migrations.length,
  ).reverse(),
);
</script>
