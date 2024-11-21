<template>
  <div class="dark:border-neutral-600 border-b">
    <label
      class="relative text-neutral-400 focus-within:text-neutral-600 block h-[34px]"
    >
      <input
        v-model="searchString"
        :placeholder="placeholder"
        :class="
          clsx(
            'w-full text-xs pl-[32px] py-2xs h-[34px] placeholder:italic',
            'focus:bg-neutral-50 focus:dark:bg-shade-100 focus:outline focus:outline-2 focus:outline-action-500 outline-offset-[-1px]',
            themeClasses(
              'text-black bg-shade-0 placeholder:text-neutral-500',
              'text-white bg-neutral-800 placeholder:text-neutral-400',
            ),
            filtersEnabled ? 'pr-[58px]' : 'pr-[30px]',
          )
        "
        @keydown="onKeyDown"
      />
      <Icon
        name="search"
        class="absolute left-2xs top-[6px] dark:text-neutral-600"
      />
      <div
        class="absolute right-0 top-0 h-[34px] flex flex-row items-center px-2xs"
      >
        <Icon
          v-if="searchString"
          name="x-circle"
          class="text-neutral-400 dark:text-neutral-500 cursor-pointer hover:text-shade-100 hover:dark:text-shade-0"
          @click="clearSearch"
        />
        <IconButton
          v-if="filtersEnabled && filters && filters.length > 0"
          icon="filter"
          :selected="showFilters"
          @click="toggleShowFilters"
        />
      </div>
    </label>

    <Transition
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
      <div v-show="showFilters" ref="transitionRef">
        <div
          class="px-xs pt-xs pb-2xs flex flex-row flex-wrap gap-2xs select-none"
        >
          <FilterPill
            v-for="(filter, index) in filters"
            :key="index"
            :filter="filter"
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
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref, watch } from "vue";
import clsx from "clsx";
import FilterPill from "./FilterPill.vue";
import Icon from "../icons/Icon.vue";
import IconButton from "./IconButton.vue";
import { Tones } from "../utils/color_utils";
import { IconNames } from "../icons/icon_set";
import { themeClasses } from "../utils/theme_tools";

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
  iconColor?: string;
  iconTone?: Tones;
  iconName?: IconNames;
  count?: number;
};

const emit = defineEmits<{
  (e: "search", searchTerm: string): void;
  (e: "update:modelValue", newValue: string): void;
}>();

const props = defineProps({
  placeholder: { type: String, default: "search" },
  modelValue: { type: String },
  filters: { type: Array<Filter> },
  disableFilters: { type: Boolean },
});

const filtersEnabled = computed(() => props.filters && !props.disableFilters);

const showFilters = ref(false);

const toggleShowFilters = () => {
  showFilters.value = !showFilters.value;
  if (!showFilters.value) resetFilters(); // if the filter panel is collapsed, reset all filters
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

watch(searchString, () => {
  emit("update:modelValue", searchString.value || "");
  debouncedAutoSearch();
});

function triggerAutoSearch() {
  triggerSearch();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") triggerSearch();
}

defineExpose({ filteringActive, activeFilters });
</script>
