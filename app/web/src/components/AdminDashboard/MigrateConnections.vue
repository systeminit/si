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
        <div class="flex flex-row gap-xs p-xs w-full">
          <div>
            <h2>Migrateable Connections: {{ migrateable.length }}</h2>
            <div v-for="migration of migrateable" :key="migration.message">
              {{ migration.message }}
            </div>
          </div>
          <div v-if="unmigrateableBecause.length > 0">
            <h2>
              Unmigrateable Connections:
              {{ unmigrateableBecause.map((migrations) => migrations.length) }}
            </h2>
            <div
              v-for="[because, migrations] in unmigrateableBecause"
              :key="because"
            >
              <h3>{{ because }}: {{ migrations.length }}</h3>
              <div v-for="migration in migrations" :key="migration.message">
                {{ migration.message }}
              </div>
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
const requestStatus = adminStore.getRequestStatus("MIGRATE_CONNECTIONS");
const allMigrations = computed(
  () => adminStore.migrateConnectionsResponse?.migrations ?? [],
);
const migrateable = computed(() =>
  allMigrations.value.filter((migration) => !migration.issue),
);
const unmigrateableBecause = computed(() =>
  _.sortBy(
    Object.entries(
      _.groupBy(
        allMigrations.value.filter((migration) => !!migration.issue),
        (migration) => migration.issue?.type,
      ),
    ),
    ([_, migrations]) => migrations.length,
  ).reverse(),
);
</script>
