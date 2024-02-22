<template>
  <!-- NEW SEARCH BAR WITH FILTERS -->
  <div
    v-if="featureFlagsStore.SEARCH_FILTERS"
    class="dark:border-neutral-600 border-b py-2xs"
  >
    <div class="flex nowrap gap-xs px-xs justify-between">
      <label
        class="relative text-neutral-400 focus-within:text-neutral-600 block flex-grow"
      >
        <input
          v-model="searchString"
          :placeholder="placeholder"
          :class="
            clsx(
              'w-full px-1 py-1 text-xs rounded',
              'border text-black dark:text-white bg-shade-0 dark:bg-shade-100 border-neutral-300 dark:border-neutral-600',
              'placeholder:italic placeholder:text-neutral-500 dark:placeholder:text-neutral-400',
              'focus:outline focus:outline-2 focus:outline-action-500 outline-offset-[-1px]',
            )
          "
          @keydown="onKeyDown"
        />
        <Icon
          name="x-circle"
          class="absolute top-1 right-1 text-neutral-400 dark:text-neutral-500 cursor-pointer hover:text-shade-100 hover:dark:text-shade-0"
          size="sm"
          @click="clearSearch"
        />
      </label>
      <button
        v-if="!disableFilters && filters"
        :class="
          clsx(
            'w-6 hover:scale-110 active:dark:text-action-300 active:text-action-500',
            showFilters
              ? 'text-action-400'
              : 'text-neutral-500 hover:text-shade-100 dark:hover:text-shade-0',
          )
        "
        @click="toggleShowFilters"
      >
        <Icon name="filter" />
      </button>
      <button
        v-if="!autoSearch || disableFilters || !filters"
        class="w-6 text-neutral-500 hover:text-shade-100 hover:dark:text-shade-0 hover:scale-110 active:dark:text-action-300 active:text-action-500"
        @click="triggerSearch"
      >
        <Icon name="search" />
      </button>
    </div>
    <Transition
      v-show="showFilters"
      name="expand-height"
      enterActiveClass="transition-[height] overflow-hidden"
      leaveActiveClass="transition-[height] overflow-hidden"
      enterFromClass="!h-0"
      leaveToClass="!h-0"
      :onBeforeEnter="captureHeight"
      :onAfterEnter="clearHeight"
      :onBeforeLeave="captureHeight"
      :onAfterLeave="clearHeight"
    >
      <div ref="transitionRef">
        <div
          class="px-xs pt-xs pb-2xs flex flex-row flex-wrap gap-2xs select-none"
        >
          <FilterPill
            v-for="(filter, index) in filters"
            :key="index"
            :label="filter.name"
            :number="filter.count"
            :iconTone="filter.iconTone"
            :iconName="filter.iconName"
            :selected="activeFilters[index]"
            @click="toggleFilter(index)"
          />
          <div
            class="flex flex-row items-center text-sm text-neutral-500 dark:text-neutral-400 hover:underline cursor-pointer active:dark:text-action-300 active:text-action-500 active:font-bold group"
            @click="resetFilters"
          >
            <Icon name="x" size="sm" class="group-active:scale-125" />
            <div>Reset</div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
  <!-- OLD SEARCH BAR WITHOUT FILTERS -->
  <div
    v-else
    class="flex nowrap gap-3 p-3 dark:border-neutral-600 border-b justify-between"
  >
    <label
      class="relative text-neutral-400 focus-within:text-neutral-600 block flex-grow"
    >
      <input
        v-model="searchString"
        :placeholder="placeholder"
        :class="
          clsx(
            'w-full px-1 py-[0.4375rem] pl-2.5 text-sm rounded-sm',
            'border text-black dark:text-white bg-neutral-50 dark:bg-neutral-900 border-neutral-300 dark:border-neutral-600',
            'placeholder:italic placeholder:text-neutral-500 dark:placeholder:text-neutral-400',
            'focus:outline focus:outline-2 focus:outline-action-500 outline-offset-[-1px]',
          )
        "
        @keydown="onKeyDown"
      />
    </label>
    <button
      class="w-6 text-action- text-neutral-500 hover:text-shade-100 dark:hover:text-shade-0 hover:scale-110"
      @click="triggerSearch"
    >
      <Icon name="search" />
    </button>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref, watch } from "vue";
import { Icon, IconNames, Tones } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import FilterPill from "./FilterPill.vue";

const transitionRef = ref<HTMLDivElement>();

const captureHeight = () => {
  if (transitionRef.value) {
    if (transitionRef.value.style.display === "none") {
      transitionRef.value.style.removeProperty("display");
    }
    transitionRef.value.style.height = `${transitionRef.value.clientHeight}px`;
  }
};
const clearHeight = () => {
  if (transitionRef.value) {
    transitionRef.value.style.height = "";
  }
};

export type Filter = {
  name: string;
  iconTone?: Tones;
  iconName?: IconNames;
  count?: number;
};

const featureFlagsStore = useFeatureFlagsStore();

const emit = defineEmits<{
  (e: "search", searchTerm: string): void;
  (e: "update:modelValue", newValue: string): void;
}>();

const props = defineProps({
  placeholder: { type: String, default: "search" },
  modelValue: { type: String },
  autoSearch: { type: Boolean },
  filters: { type: Array<Filter> },
  disableFilters: { type: Boolean },
});

const showFilters = ref(false);

const toggleShowFilters = () => {
  showFilters.value = !showFilters.value;
};

const activeFilters = ref(
  Array.from({ length: props.filters?.length || 0 }, () => false),
);

const filteringActive = computed(() =>
  activeFilters.value.some((filter) => filter),
);

const resetFilters = () => {
  for (let i = 0; i < activeFilters.value.length; i++) {
    activeFilters.value[i] = false;
  }
};

const toggleFilter = (index: number) => {
  if (index < 0 || index >= activeFilters.value.length) return;

  activeFilters.value[index] = !activeFilters.value[index];

  if (filteringActive.value) {
    emit("search", searchString.value || "");
  }
};

const searchString = ref(props.modelValue) || "";
watch(
  () => props.modelValue,
  () => {
    searchString.value = props.modelValue;
  },
);

function triggerSearch() {
  emit("search", searchString.value || "");
}

function clearSearch() {
  searchString.value = "";
}

const debouncedAutoSearch = _.debounce(triggerAutoSearch, 50);

// if autoSearch prop is true, we'll trigger the search event as the user types (debounced)
// rather than only when they click the search icon
watch(searchString, () => {
  emit("update:modelValue", searchString.value || "");
  debouncedAutoSearch();
});

function triggerAutoSearch() {
  if (props.autoSearch) triggerSearch();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") triggerSearch();
}

defineExpose({ filteringActive, activeFilters });
</script>
