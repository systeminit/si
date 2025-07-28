<template>
  <div class="flex flex-col min-h-full">
    <div
      v-if="showSubheader"
      :class="clsx('header flex flex-col px-2xs pb-2xs')"
    >
      <div class="flex flex-row justify-between items-center"></div>
      <div
        :class="
          clsx(
            'flex flex-row items-center gap-sm flex-wrap border-l border-r border-b py-2xs -mx-2xs pl-xs pr-2xs',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <DropdownMenuButton
          class="rounded min-w-[120px]"
          :options="funcKindOptions"
          :modelValue="selectedFuncKind"
          placeholder="Filter by Kind"
          minWidthToAnchor
          checkable
          alwaysShowPlaceholder
          @update:modelValue="(val) => (selectedFuncKind = val)"
        >
          <template #beforeOptions>
            <DropdownMenuItem
              label="All Kinds"
              value=""
              checkable
              :checked="selectedFuncKind === ''"
              @select="() => (selectedFuncKind = '')"
            />
          </template>
        </DropdownMenuButton>
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
import {
  Icon,
  themeClasses,
  DropdownMenuButton,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
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

const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } =
  useInfiniteQuery({
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

// Convert func kinds to dropdown options format
const funcKindOptions = computed(() => {
  return availableFuncKinds.value.map((kind) => ({
    value: kind,
    label: kind,
  }));
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
