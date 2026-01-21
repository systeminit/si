<template>
  <main v-if="groups && groups.length > 0">
    <header
      :class="
        clsx(
          'flex flex-row items-center gap-xs px-sm py-xs border-t border-b border-neutral-600',
          themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
    >
      <h1 class="text-lg">Policies</h1>
    </header>
    <section class="grid gap-sm grid-cols-3 xl:grid-cols-5 p-sm">
      <div
        v-for="group in groups"
        :key="group.name"
        :class="clsx('border', themeClasses('border-neutral-200', 'border-neutral-600'))"
      >
        <h3
          :class="
            clsx(
              'p-sm border-b text-md',
              themeClasses('border-neutral-200 bg-neutral-200', 'border-neutral-600 bg-neutral-800'),
            )
          "
        >
          {{ group.name }}
        </h3>
        <ol class="m-xs text-sm">
          <li
            v-for="p in group.results"
            :key="p.id"
            :class="
              clsx(
                'p-xs cursor-pointer border mb-xs',
                themeClasses(
                  'hover:border-action-500 border-neutral-200',
                  'hover:border-action-300 border-neutral-600',
                ),
                'flex flex-row items-center text-xs gap-sm justify-between',
              )
            "
            @click="() => navigateToPolicy(p)"
          >
            <Icon
              class="shrink"
              size="xs"
              :name="p.result === 'Fail' ? 'triangle' : 'check-square'"
              :tone="p.result === 'Fail' ? 'destructive' : 'success'"
            />
            <span class="grow flex flex-row items-center">
              <Icon name="git-branch" tone="neutral" size="xs" />
              <TruncateWithTooltip class="py-2xs">{{ p.changeSetName }}</TruncateWithTooltip>
            </span>
            <span class="shrink">
              <Timestamp refresh size="normal" relative="standard" showTimeIfToday :date="p.createdAt" />
            </span>
          </li>
        </ol>
      </div>
    </section>
  </main>
  <main v-else-if="policyQuery.isFetched.value && groups?.length === 0">
    <EmptyStateCard
      iconName="no-changes"
      primaryText="No Policy Reports have been run"
      secondaryText="See our How-To guides to evaluate policies"
    />
  </main>
  <main v-else-if="policyQuery.isFetched.value">
    <EmptyStateCard
      iconName="no-changes"
      primaryText="No Policy Reports found"
      secondaryText="See our How-To guides to evaluate policies"
    />
  </main>
  <main v-else-if="policyQuery.isLoading.value">
    <DelayedLoader size="full" />
  </main>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { routes, useApi } from "./api_composables";
import { useContext } from "./logic_composables/context";
import { useQuery } from "@tanstack/vue-query";
import { Policy } from "./logic_composables/policy";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import clsx from "clsx";
import { themeClasses, Icon, Timestamp, TruncateWithTooltip } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import DelayedLoader from "@/newhotness/layout_components/DelayedLoader.vue";

const router = useRouter();
const route = useRoute();

interface GroupedPolicy {
  name: string;
  results: Policy[];
}

const ctx = useContext();
const api = useApi(ctx);
const queryKey = computed(() => ["policies", "groups"]);
const policyQuery = useQuery({
  enabled: true,
  queryKey,
  staleTime: 5000,
  queryFn: async () => {
    const call = api.endpoint<{ groups: GroupedPolicy[] | null }>(routes.GroupedPolicyReports, {});
    const response = await call.get();
    if (api.ok(response)) {
      return response.data.groups;
    }
    return null;
  },
});

const groups = computed<GroupedPolicy[] | null>(() => {
  return policyQuery.data.value || null;
});

const navigateToPolicy = (policy: Policy) => {
  router.push({
    name: "new-hotness-policy",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      policyId: policy.id,
      policyName: policy.name,
    },
  });
};
</script>
