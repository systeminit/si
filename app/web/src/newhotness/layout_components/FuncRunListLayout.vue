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

        <!-- Slot for additional filters (like component name filter) -->
        <slot name="additional-filters" />

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
          :text="emptyStateText || ''"
          :secondaryText="emptyStateSecondaryText"
          class="max-w-[420px]"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import {
  Icon,
  themeClasses,
  DropdownMenuButton,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import FuncRunCard from "../FuncRunCard.vue";
import { FuncRun } from "../api_composables/func_run";
import EmptyState from "../EmptyState.vue";

// Props
const props = defineProps<{
  funcRuns: FuncRun[];
  availableFuncKinds: string[];
  selectedFuncKind: string;
  hasActiveFilters: boolean;
  isFetchingNextPage: boolean;
  hasNextPage: boolean;
  isLoading: boolean;
  emptyStateText?: string;
  emptyStateSecondaryText?: string;
}>();

// Emits
const emit = defineEmits<{
  (e: "update:selectedFuncKind", value: string): void;
  (e: "resetFilters"): void;
  (e: "scroll", event: Event): void;
}>();

// Router setup
const router = useRouter();
const route = useRoute();

// Scroll container reference for infinite loading
const scrollContainerRef = ref<HTMLElement | null>(null);

// Convert func kinds to dropdown options format
const funcKindOptions = computed(() => {
  return props.availableFuncKinds.map((kind) => ({
    value: kind,
    label: kind,
  }));
});

// Show subheader if there are func runs or currently loading
const showSubheader = computed(
  () => props.funcRuns.length > 0 || props.isLoading,
);

// Handle scroll to implement infinite loading
const handleScroll = (event: Event) => {
  emit("scroll", event);
};

// Handle filter updates
const selectedFuncKind = computed({
  get: () => props.selectedFuncKind,
  set: (value: string) => emit("update:selectedFuncKind", value),
});

// Reset filters
const resetFilters = () => {
  emit("resetFilters");
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
