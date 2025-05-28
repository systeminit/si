<template>
  <div class="flex flex-col min-h-0">
    <div
      :class="
        clsx(
          'header flex flex-row justify-between items-center p-2xs',
          themeClasses('bg-neutral-200', 'bg-neutral-900'),
        )
      "
    >
      <div class="text-sm font-medium">Recent Function Runs</div>
      <div
        v-if="isFetching && !isFetchingNextPage"
        class="text-xs text-neutral-500 flex items-center"
      >
        <Icon name="loader" size="xs" class="animate-spin mr-1" />
        Updating...
      </div>
    </div>

    <div ref="scrollContainerRef" class="scrollable" @scroll="handleScroll">
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
        class="py-4 text-center text-neutral-500"
      >
        No function runs found
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onBeforeUnmount, inject } from "vue";
import { useInfiniteQuery, useQueryClient } from "@tanstack/vue-query";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import { FuncRun } from "@/store/func_runs.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import FuncRunCard from "./FuncRunCard.vue";
import { funcRunTypes, useApi, routes } from "./api_composables";
import { assertIsDefined, Context } from "./types";

// Component props
const props = defineProps<{
  limit?: number;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// Router setup
const router = useRouter();
const route = useRoute();

// Query client for cache management
const queryClient = useQueryClient();

// Realtime store for WSEvents
const realtimeStore = useRealtimeStore();

// Configure page size with default fallback
const pageSize = computed(() => props.limit || 50);

// Scroll container reference for infinite loading
const scrollContainerRef = ref<HTMLElement | null>(null);

const api = useApi();

const {
  data,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isFetching,
  isLoading,
} = useInfiniteQuery({
  queryKey: [ctx.changeSetId.value, "paginatedFuncRuns"],
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
const funcRuns = computed<FuncRun[]>(() => {
  if (!data.value) return [];
  return data.value.pages.flatMap((page) => page.funcRuns);
});

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

// WSEvents subscription to invalidate cache when function runs are updated
let executionKey: string | undefined;

// Set up subscription on mount
onMounted(async () => {
  realtimeStore.subscribe("paginatedFuncRuns", `changeset/${ctx.changeSetId}`, [
    {
      eventType: "FuncRunLogUpdated",
      callback: async (payload) => {
        if (payload.funcRunId) {
          queryClient.invalidateQueries({
            queryKey: [ctx.changeSetId.value, "paginatedFuncRuns"],
          });
        }
      },
    },
  ]);
});

// Clean up on unmount
onBeforeUnmount(() => {
  if (executionKey) {
    // realtimeStore.unsubscribe(executionKey);
    executionKey = undefined;
  }
});
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
