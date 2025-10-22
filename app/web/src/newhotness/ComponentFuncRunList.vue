<template>
  <FuncRunListLayout
    :funcRuns="funcRuns"
    :availableFuncKinds="availableFuncKinds"
    :selectedFuncKind="selectedFuncKind"
    :hasActiveFilters="hasActiveFilters"
    :isFetchingNextPage="isFetchingNextPage"
    :hasNextPage="hasNextPage"
    :isLoading="isLoading"
    emptyStateText="No function runs yet"
    emptyStateSecondaryText="Function history shows the output of executed functions for this component, including logs, generated code, passed arguments, and results."
    @update:selectedFuncKind="(val) => (selectedFuncKind = val)"
    @resetFilters="resetFilters"
    @scroll="handleScroll"
  />
</template>

<script lang="ts" setup>
import { computed, ref, inject } from "vue";
import { useInfiniteQuery } from "@tanstack/vue-query";
import FuncRunListLayout from "./layout_components/FuncRunListLayout.vue";
import { funcRunTypes, useApi, routes } from "./api_composables";
import { assertIsDefined, Context } from "./types";
import { FuncRun } from "./api_composables/func_run";
// Component props
const props = defineProps<{
  componentId: string;
  limit?: number;
  enabled?: boolean;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// Configure page size with default fallback
const pageSize = computed(() => props.limit || 50);

// Filter state
const selectedFuncKind = ref<string>("");

const api = useApi();

const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } =
  useInfiniteQuery({
    queryKey: [ctx.changeSetId, "paginatedFuncRuns", props.componentId],
    enabled: computed(() => props.enabled ?? true),
    queryFn: async ({
      pageParam = undefined,
    }): Promise<funcRunTypes.GetFuncRunsPaginatedResponse> => {
      const call = api.endpoint<funcRunTypes.GetFuncRunsPaginatedResponse>(
        routes.GetFuncRunsPaginated,
      );
      const params = new URLSearchParams();
      params.append("limit", pageSize.value.toString());
      params.append("componentId", props.componentId);
      if (pageParam) {
        params.append("cursor", pageParam);
      }
      const req = await call.get(params);
      if (api.ok(req)) {
        return req.data;
      }
      return {
        funcRuns: [],
        nextCursor: null,
      };
    },
    initialPageParam: undefined,
    getNextPageParam: (lastPage: funcRunTypes.GetFuncRunsPaginatedResponse) => {
      return lastPage.nextCursor ?? undefined;
    },
  });

// Flatten the pages of function runs for display
const allFuncRuns = computed<FuncRun[]>(() => {
  if (!data.value) return [];
  return data.value.pages.flatMap((page) => page.funcRuns);
});

// Get available func kinds from the data
const availableFuncKinds = computed(() => {
  const kinds = new Set<string>();
  allFuncRuns.value.forEach((funcRun) => {
    if (funcRun.functionKind) {
      kinds.add(funcRun.functionKind);
    }
  });
  return Array.from(kinds).sort();
});

// Filter function runs based on selected func kind
const funcRuns = computed<FuncRun[]>(() => {
  let filtered = allFuncRuns.value;

  // Filter by function kind if selected
  if (selectedFuncKind.value) {
    filtered = filtered.filter(
      (funcRun) => funcRun.functionKind === selectedFuncKind.value,
    );
  }

  return filtered;
});

// Check if any filters are active
const hasActiveFilters = computed(() => {
  return selectedFuncKind.value !== "";
});

// Reset all filters
const resetFilters = () => {
  selectedFuncKind.value = "";
};

// Handle scroll to implement infinite loading
const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement;
  if (!target) return;

  const { scrollTop, scrollHeight, clientHeight } = target;
  const scrollBottom = scrollHeight - scrollTop - clientHeight;

  // Load more when user scrolls near the bottom (within 200px)
  if (scrollBottom < 200 && hasNextPage.value && !isFetchingNextPage.value) {
    fetchNextPage();
  }
};
</script>
