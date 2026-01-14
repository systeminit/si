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
    emptyStateSecondaryText="Function history shows the output of executed functions, including logs, generated code, passed arguments, and results."
    @update:selectedFuncKind="(val) => (selectedFuncKind = val)"
    @resetFilters="resetFilters"
    @scroll="handleScroll"
  >
    <template #additional-filters>
      <div
        :class="
          clsx(
            'flex flex-row items-center gap-2xs border rounded',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <input
          v-model="componentNameFilter"
          type="text"
          placeholder="Find by component name"
          :class="
            clsx(
              'text-xs px-2xs py-2xs min-w-[250px] border-0',
              themeClasses(
                'bg-white text-neutral-600 placeholder-neutral-500',
                'bg-black text-neutral-400 placeholder-neutral-400',
              ),
            )
          "
        />
      </div>
    </template>
  </FuncRunListLayout>
</template>

<script lang="ts" setup>
import { computed, ref, inject } from "vue";
import { useInfiniteQuery } from "@tanstack/vue-query";
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import FuncRunListLayout from "./layout_components/FuncRunListLayout.vue";
import { funcRunTypes, useApi, routes } from "./api_composables";
import { assertIsDefined, Context } from "./types";
import { FuncRun } from "./api_composables/func_run";

// Component props
const props = defineProps<{
  limit?: number;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// Configure page size with default fallback
const pageSize = computed(() => props.limit || 50);

// Filter state
const selectedFuncKind = ref<string>("");
const componentNameFilter = ref<string>("");

const api = useApi();

const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } = useInfiniteQuery({
  queryKey: [ctx.changeSetId, "paginatedFuncRuns"],
  queryFn: async ({ pageParam = undefined }): Promise<funcRunTypes.GetFuncRunsPaginatedResponse> => {
    const call = api.endpoint<funcRunTypes.GetFuncRunsPaginatedResponse>(routes.GetFuncRunsPaginated);
    const params = new URLSearchParams();
    params.append("limit", pageSize.value.toString());
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

// Filter function runs based on selected func kind and component name
const funcRuns = computed<FuncRun[]>(() => {
  let filtered = allFuncRuns.value;

  // Filter by function kind if selected
  if (selectedFuncKind.value) {
    filtered = filtered.filter(
      // eslint-disable-next-line @typescript-eslint/no-unsafe-enum-comparison
      (funcRun) => funcRun.functionKind === selectedFuncKind.value,
    );
  }

  // Filter by component name if provided
  if (componentNameFilter.value.trim()) {
    const searchTerm = componentNameFilter.value.toLowerCase().trim();
    filtered = filtered.filter((funcRun) => {
      return funcRun.componentName?.toLowerCase().includes(searchTerm) || false;
    });
  }

  return filtered;
});

// Check if any filters are active
const hasActiveFilters = computed(() => {
  return selectedFuncKind.value !== "" || componentNameFilter.value.trim() !== "";
});

// Reset all filters
const resetFilters = () => {
  selectedFuncKind.value = "";
  componentNameFilter.value = "";
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
