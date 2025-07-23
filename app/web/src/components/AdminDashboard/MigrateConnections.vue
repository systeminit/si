<template>
  <Stack class="p-10 w-full">
    <h1>Migrate Connections {{ migrationRun?.dryRun ? "(dry run)" : "" }}</h1>

    <!-- Show status -->
    <div v-if="!migrationRun">Waiting for migrationRun to start ...</div>
    <div v-else-if="!migrationRun.summary" class="text-neutral-500">
      Migrating connections ... ({{ elapsed }})
    </div>
    <div v-else-if="migrationRun.summary.error" class="text-red-500">
      Error migrating after {{ elapsed }}: {{ migrationRun.summary.error }}
    </div>
    <div v-else>
      Migrated {{ migrationRun?.dryRun ? "(dry run)" : "" }} in {{ elapsed }}!
    </div>

    <!-- Show migrations -->
    <Stack class="flex flex-col gap-xs p-xs w-full font-bold text-xs">
      <Stack>
        <div class="text-lg">
          {{ migrationRun?.dryRun ? "Migrateable" : "Migrated" }} Connections:
          {{ migrateable.length }}
        </div>
        <pre v-for="migration of migrateable" :key="migration.message">{{
          migration.message.replaceAll(" | ", "\n")
        }}</pre>
      </Stack>
      <Stack v-if="unmigrateableBecause.length > 0">
        <div class="text-lg">
          Unmigrateable Connections:
          {{
            unmigrateableBecause
              .map(([_, migrations]) => migrations.length)
              .reduce((a, b) => a + b, 0)
          }}
        </div>
        <div
          v-for="[because, migrations] in unmigrateableBecause"
          :key="because"
        >
          <div class="text-lg">{{ because }}: {{ migrations.length }}</div>
          <div v-for="migration in migrations" :key="migration.message">
            {{ migration.message.replaceAll(" | ", "\n  ") }}
          </div>
        </div>
      </Stack>
    </Stack>
  </Stack>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { Stack, durationString } from "@si/vue-lib/design-system";
import { useNow } from "@vueuse/core";
import {
  ConnectionMigration,
  ConnectionUnmigrateableBecause,
  useAdminStore,
} from "@/store/admin.store";

const adminStore = useAdminStore();
const migrationRun = computed(() => adminStore.connectionMigrationRun);
const now = useNow();
const elapsed = computed(() => {
  if (!migrationRun.value) return "<error: not started>";
  const start = migrationRun.value.startedAt;
  const end = migrationRun.value.summary?.finishedAt ?? now.value;
  return durationString(end.getTime() - start.getTime());
});

const migrationsByIssue = computed(
  () =>
    _.groupBy(
      migrationRun.value?.migrations ?? [],
      (migrationRun) => migrationRun.issue?.type ?? "migrateable",
    ) as Record<
      ConnectionUnmigrateableBecause["type"] | "migrateable",
      ConnectionMigration[]
    >,
);
const migrateable = computed(() => migrationsByIssue.value.migrateable ?? []);
const unmigrateableBecause = computed(() =>
  _.sortBy(
    Object.entries(migrationsByIssue.value),
    ([_, migrations]) => migrations.length,
  )
    .reverse()
    .filter(([because, _]) => because !== "migrateable"),
);
</script>
