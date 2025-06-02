<template>
  <Stack class="p-10 w-full">
    <h1>Migrate Connections</h1>
    <LoadStatus :requestStatus="requestStatus">
      <template #loading>
        <div class="text-neutral-500">Migrating connections ...</div>
      </template>
      <template #error>
        <div class="text-red-500">
          Error migrating connections: {{ requestStatus.errorMessage }}
        </div>
      </template>
      <template #success>
        <Stack class="flex flex-col gap-xs p-xs w-full font-bold text-xs">
          <Stack>
            <div class="text-lg">
              Migrateable Connections: {{ migrateable.length }}
            </div>
            <div v-for="migration of migrateable" :key="migration.message">
              {{ migration.message }}
            </div>
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
                {{ migration.message }}
              </div>
            </div>
          </Stack>
          <Stack
            v-if="alreadyMigrated.length > 0"
            class="flex flex-row gap-xs p-xs w-full"
          >
            <div class="text-lg">
              Already Migrated Connections: {{ alreadyMigrated.length }}
            </div>
            <div v-for="migration of alreadyMigrated" :key="migration.message">
              {{ migration.message }}
            </div>
          </Stack>
        </Stack>
      </template>
    </LoadStatus>
  </Stack>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { Stack, LoadStatus } from "@si/vue-lib/design-system";
import {
  ConnectionMigration,
  ConnectionUnmigrateableBecause,
  useAdminStore,
} from "@/store/admin.store";

const adminStore = useAdminStore();
const requestStatus = adminStore.getRequestStatus("MIGRATE_CONNECTIONS");
const migrationsByIssue = computed(
  () =>
    _.groupBy(
      adminStore.migrateConnectionsResponse?.migrations ?? [],
      (migration) => migration.issue?.type ?? "migrateable",
    ) as Record<
      ConnectionUnmigrateableBecause["type"] | "migrateable",
      ConnectionMigration[]
    >,
);
const migrateable = computed(() => migrationsByIssue.value.migrateable ?? []);
const alreadyMigrated = computed(
  () => migrationsByIssue.value.destinationPropAlreadyHasValue ?? [],
);
const unmigrateableBecause = computed(() =>
  _.sortBy(
    Object.entries(migrationsByIssue.value),
    ([_, migrations]) => migrations.length,
  )
    .reverse()
    .filter(
      ([because, _]) =>
        because !== "migrateable" &&
        because !== "destinationPropAlreadyHasValue",
    ),
);
</script>
