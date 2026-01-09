<template>
  <div class="h-full overflow-y-auto pb-md">
    <header
      :class="
        clsx(
          'flex flex-row items-center gap-xs px-sm py-xs border-t border-b border-neutral-600',
          themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
    >
      <NewButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        icon="x"
        tone="empty"
        :class="
          clsx(
            'active:bg-white active:text-black',
            themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'),
          )
        "
        @click="navigateBack"
      />
      <template v-if="policy">
        <div
          class="shrink ml-auto mr-auto flex flex-row gap-sm items-center justify-center min-w-[32vw]"
        >
          <Icon
            class="shrink"
            size="xs"
            :name="policy.result === 'Fail' ? 'triangle' : 'check-square'"
            :tone="policy.result === 'Fail' ? 'destructive' : 'success'"
          />
          <span class="grow"
            ><TruncateWithTooltip class="py-2xs">{{
              policy.name
            }}</TruncateWithTooltip></span
          >
          <span class="shrink">
            <Timestamp
              refresh
              size="normal"
              relative="standard"
              showTimeIfToday
              :date="policy.createdAt"
            />
          </span>
        </div>
      </template>
    </header>
    <div class="w-[70vw] ml-auto mr-auto flex flex-row">
      <div
        :class="
          clsx(
            'w-[25vw] pt-md pr-md border-r-[1px]',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <h5 class="mb-sm text-sm ml-xs">Policy history</h5>
        <PolicyList
          :policies="policyReports"
          :page="page"
          :maxPages="maxPages"
          @pageBack="pageBack"
          @pageForward="pageForward"
          @select="(p) => navigateToPolicy(p)"
        />
      </div>
      <div v-if="!policy" class="w-full">
        <EmptyStateCard
          iconName="no-changes"
          primaryText="Could not find this Policy Report"
          secondaryText="Please select another from the list of policy reports."
        />
      </div>
      <section v-else class="w-full p-md">
        <div class="mb-lg">
          <MarkdownRender :source="policy.policy" />
        </div>
        <div
          :class="
            clsx(
              'border-t-[1px] pt-lg',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            )
          "
        >
          <MarkdownRender :source="policy.report" />
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { clsx } from "clsx";
import { useRoute, useRouter } from "vue-router";
import { computed, watch } from "vue";
import {
  themeClasses,
  NewButton,
  Icon,
  TruncateWithTooltip,
  Timestamp,
} from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { Policy, usePolicy } from "./logic_composables/policy";
import PolicyList from "./layout_components/PolicyList.vue";
import MarkdownRender from "./MarkdownRender.vue";
import { routes, useApi } from "./api_composables";
import { useContext } from "./logic_composables/context";
import { prevPage } from "./logic_composables/navigation_stack";

const router = useRouter();
const route = useRoute();

// Navigate back to explore_grid view
const navigateBack = () => {
  const lastPage = prevPage();
  // if we aren't coming from new-hotness, go to where we came from
  // usually component details
  if (lastPage && lastPage.name !== "new-hotness") {
    router.push({
      name: lastPage.name,
      params: lastPage.params,
    });
  } else {
    router.push({
      name: "new-hotness",
      params: {
        workspacePk: route.params.workspacePk,
        changeSetId: route.params.changeSetId,
      },
      query: { retainSessionState: 1 },
    });
  }
};

const props = defineProps<{
  policyId: string;
}>();

const { policyReports, page, maxPages } = usePolicy();

watch(
  route.query,
  () => {
    if (!route.query.page) return;

    const urlPage = parseInt(route.query.page?.toString() || "1");
    if (urlPage && urlPage !== page.value) page.value = urlPage;
  },
  { immediate: true },
);

watch(
  page,
  () => {
    const urlPage = parseInt(route.query.page?.toString() || "1");
    if (urlPage !== page.value) {
      router.push({
        ...route.params,
        query: { page: page.value },
      });
    }
  },
  { immediate: true },
);

const pageBack = () => {
  if (page.value === 1) page.value = maxPages.value;
  else page.value -= 1;
};
const pageForward = () => {
  if (page.value === maxPages.value) page.value = 1;
  else page.value += 1;
};

const navigateToPolicy = (policy: Policy) => {
  router.push({
    name: "new-hotness-policy",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      policyId: policy.id,
    },
    query: {
      page: page.value,
    },
  });
};

const ctx = useContext();
const api = useApi(ctx);
const queryKey = computed(() => ["policies", props.policyId]);
const policyQuery = useQuery<Policy | null>({
  enabled: true,
  queryKey,
  staleTime: 5000,
  queryFn: async () => {
    const call = api.endpoint<{ report: Policy | null }>(routes.PolicyReport, {
      policyId: props.policyId,
    });
    const response = await call.get();
    if (api.ok(response)) {
      return response.data.report;
    }
    return null;
  },
});

const policy = computed<Policy | null>(() => {
  return policyQuery.data.value || null;
});
</script>
