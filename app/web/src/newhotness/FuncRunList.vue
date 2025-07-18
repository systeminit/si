<template>
  <div class="flex flex-col min-h-full">
    <div
      v-if="showSubheader"
      :class="
        clsx(
          'header flex flex-col gap-2xs p-2xs',
          themeClasses('bg-neutral-200', 'bg-neutral-900'),
        )
      "
    >
      <div class="flex flex-row justify-between items-center">
        <div class="text-sm font-medium">Recent Function Runs</div>
        <div
          v-if="isFetching && !isFetchingNextPage"
          class="text-xs text-neutral-500 flex items-center"
        >
          <Icon name="loader" size="xs" class="animate-spin mr-1" />
          Updating...
        </div>
      </div>
      <div class="flex flex-row items-center gap-sm flex-wrap">
        <div class="flex flex-row items-center gap-2xs">
          <select
            v-model="selectedFuncKind"
            :class="
              clsx(
                'text-xs border rounded px-2xs py-1 min-w-[120px]',
                themeClasses(
                  'bg-shade-0 border-neutral-400 text-shade-100',
                  'bg-neutral-800 border-neutral-600 text-shade-0',
                ),
              )
            "
          >
            <option value="" disabled>Filter by Kind</option>
            <option
              v-for="kind in availableFuncKinds"
              :key="kind"
              :value="kind"
            >
              {{ kind }}
            </option>
          </select>
        </div>
        <div class="flex flex-row items-center gap-2xs">
          <input
            v-model="componentNameFilter"
            type="text"
            placeholder="Filter by component name"
            :class="
              clsx(
                'text-xs border rounded px-2xs py-1 min-w-[250px]',
                themeClasses(
                  'bg-shade-0 border-neutral-400 text-shade-100 placeholder-neutral-500',
                  'bg-neutral-800 border-neutral-600 text-shade-0 placeholder-neutral-400',
                ),
              )
            "
          />
        </div>
        <button
          v-if="hasActiveFilters"
          :class="
            clsx(
              'text-xs border rounded px-2xs py-1 flex items-center gap-1',
              themeClasses(
                'bg-neutral-100 border-neutral-400 text-neutral-700 hover:bg-neutral-200',
                'bg-neutral-700 border-neutral-600 text-neutral-300 hover:bg-neutral-600',
              ),
            )
          "
          @click="resetFilters"
        >
          <Icon name="x" size="xs" />
          Reset
        </button>
      </div>
    </div>

    <div
      ref="scrollContainerRef"
      :class="
        clsx(
          'scrollable',
          showSubheader ? 'min-h-[calc(100%-28px)]' : 'min-h-full',
        )
      "
      @scroll="handleScroll"
    >
      <TransitionGroup name="func-run-item">
        <FuncRunCard
          v-for="funcRun in funcRuns"
          :key="funcRun.id"
          :funcRun="funcRun"
          @click="navigateToFuncRunDetails(funcRun.id)"
        />
      </TransitionGroup>

      <!-- Loading indicator at bottom -->
      <div
        v-if="isFetchingNextPage"
        class="py-2 text-center text-sm text-neutral-500"
      >
        <Icon name="loader" size="sm" class="animate-spin mr-1" />
        Loading more...
      </div>

      <!-- End of list message -->
      <div
        v-if="!hasNextPage && funcRuns.length > 0"
        class="py-2 text-center text-sm text-neutral-500"
      >
        No more function runs
      </div>

      <!-- Empty state -->
      <div
        v-if="funcRuns.length === 0 && !isLoading"
        :class="
          clsx(
            'flex flex-row items-center justify-center',
            'm-xs p-xs border min-h-[calc(100%-16px)]',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <EmptyState
          icon="func"
          text="No function runs yet"
          secondaryText="Function history shows the output of executed functions, including logs, generated code, passed arguments, and results."
          class="max-w-[420px]"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, inject } from "vue";
import { useInfiniteQuery } from "@tanstack/vue-query";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import FuncRunCard from "./FuncRunCard.vue";
import { funcRunTypes, useApi, routes } from "./api_composables";
import { assertIsDefined, Context } from "./types";
import { FuncRun } from "./api_composables/func_run";
import EmptyState from "./EmptyState.vue";

// Component props
const props = defineProps<{
  limit?: number;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// Router setup
const router = useRouter();
const route = useRoute();

// Configure page size with default fallback
const pageSize = computed(() => props.limit || 50);

// Scroll container reference for infinite loading
const scrollContainerRef = ref<HTMLElement | null>(null);

// Filter state
const selectedFuncKind = ref<string>("");
const componentNameFilter = ref<string>("");

const api = useApi();

const {
  data,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isFetching,
  isLoading,
} = useInfiniteQuery({
  queryKey: [ctx.changeSetId, "paginatedFuncRuns"],
  queryFn: async ({
    pageParam = undefined,
  }): Promise<funcRunTypes.GetFuncRunsPaginatedResponse> => {
    const call = api.endpoint<funcRunTypes.GetFuncRunsPaginatedResponse>(
      routes.GetFuncRunsPaginated,
    );
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
  return (
    selectedFuncKind.value !== "" || componentNameFilter.value.trim() !== ""
  );
});

// Reset all filters
const resetFilters = () => {
  selectedFuncKind.value = "";
  componentNameFilter.value = "";
};

// Handle scroll to implement infinite loading
const handleScroll = () => {
  if (!scrollContainerRef.value) return;

  const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef.value;
  const scrollBottom = scrollHeight - scrollTop - clientHeight;

  // Load more when user scrolls near the bottom (within 200px)
  if (scrollBottom < 200 && hasNextPage.value && !isFetchingNextPage.value) {
    fetchNextPage();
  }
};

// Navigate to FuncRunDetails when clicking a card
const navigateToFuncRunDetails = (funcRunId: string) => {
  router.push({
    name: "new-hotness-func-run",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      funcRunId,
    },
  });
};

const showSubheader = computed(
  () => allFuncRuns.value.length > 0 || isLoading.value,
);
</script>

<style>
/* Tailwind can't handle Vue transition classes directly, so we keep minimal transition styles */
.func-run-item-enter-active,
.func-run-item-leave-active {
  transition: all 0.3s ease;
}
.func-run-item-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}
.func-run-item-leave-to {
  opacity: 0;
  transform: translateY(20px);
}
</style>
