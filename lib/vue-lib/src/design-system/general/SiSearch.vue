<template>
  <div
    :class="
      clsx(
        'siSearchRoot',
        variant === 'dropdownmenu'
          ? 'rounded-t-md'
          : borderBottom && 'dark:border-neutral-600 border-b',
      )
    "
  >
    <label
      :class="
        clsx(
          'relative text-neutral-400 focus-within:text-neutral-600 block h-[34px]',
          variant === 'dropdownmenu' && 'rounded-t-md',
        )
      "
    >
      <!-- data-1p-ignore is used to prevent 1password from targeting this field -->
      <input
        ref="searchInputRef"
        v-model="searchString"
        :placeholder="placeholder"
        :class="
          clsx(
            'w-full text-xs pl-[32px] py-2xs h-[34px] placeholder:italic',
            variant === 'standard' &&
              'outline-offset-[-1px] focus:outline focus:outline-2',
            {
              dropdownmenu:
                'text-white bg-shade-100 placeholder:text-neutral-400 rounded-t-md focus:outline-none',
              standard: themeClasses(
                'text-black bg-shade-0 placeholder:text-neutral-500 focus:bg-neutral-50 focus:outline-action-500',
                'text-white bg-neutral-800 placeholder:text-neutral-400 focus:bg-shade-100 focus:outline-action-300',
              ),
              new: [
                'border outline-none focus:outline-none',
                themeClasses(
                  'text-black bg-shade-0 placeholder:text-neutral-500 border-neutral-400 focus:border-action-500',
                  'text-white bg-shade-100 placeholder:text-neutral-400 border-neutral-600 focus:border-action-300',
                ),
              ],
            }[variant],
            filtersEnabled ? 'pr-[58px]' : 'pr-[30px]',
          )
        "
        data-lpignore="true"
        data-1p-ignore
        data-bwignore
        data-form-type="other"
        :tabindex="tabIndex"
        @input="onChange"
        @keydown="onKeyDown"
        @focus="onFocus"
        @blur="onBlur"
      />
      <Icon
        name="search"
        :class="
          clsx(
            'absolute left-2xs top-[6px]',
            variant === 'dropdownmenu'
              ? 'text-neutral-600'
              : 'dark:text-neutral-600',
          )
        "
      />
      <div
        class="absolute right-0 top-0 h-[34px] flex flex-row items-center px-2xs"
      >
        <slot name="right" />
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
            v-if="allFilter"
            :filter="allFilter"
            :selected="!filteringActive"
            @click="resetFilters"
          />
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
import { computed, PropType, ref, watch } from "vue";
import clsx from "clsx";
import FilterPill from "./FilterPill.vue";
import Icon from "../icons/Icon.vue";
import IconButton from "./IconButton.vue";
import { Tones } from "../utils/color_utils";
import { IconNames } from "../icons/icon_set";
import { themeClasses } from "../utils/theme_tools";

const transitionRef = ref<HTMLDivElement>();
const searchInputRef = ref<HTMLInputElement>();

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

export type SearchVariant = "standard" | "dropdownmenu" | "new";

const emit = defineEmits<{
  (e: "search", searchTerm: string): void;
  (e: "clearSearch"): void;
  (e: "update:modelValue", newValue: string): void;
  (e: "enterPressed"): void;
  (e: "blur", event: FocusEvent): void;
  (e: "focus", event: FocusEvent): void;
  (e: "input", event: Event): void;
}>();

const props = defineProps({
  placeholder: { type: String, default: "search" },
  modelValue: { type: String },
  filters: { type: Array<Filter> },
  disableFilters: { type: Boolean },
  variant: { type: String as PropType<SearchVariant>, default: "standard" },
  borderBottom: { type: Boolean, default: true },
  allFilter: { type: Object as PropType<Filter> },
  tabIndex: { type: Number },
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
  emit("clearSearch");
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
  if (e.key === "Enter") {
    triggerSearch();
    emit("enterPressed");
  }
  if (e.key === "Escape" && searchInputRef.value) {
    searchInputRef.value.blur();
  }
}

const focusSearch = () => {
  if (searchInputRef.value) {
    searchInputRef.value.focus();
  }
};

const blurSearch = () => {
  if (searchInputRef.value) {
    searchInputRef.value.blur();
  }
};

const onBlur = (e: FocusEvent) => {
  emit("blur", e);
};

const onFocus = (e: FocusEvent) => {
  emit("focus", e);
};

const onChange = (e: Event) => {
  emit("input", e);
};

defineExpose({
  filteringActive,
  activeFilters,
  clearSearch,
  focusSearch,
  blurSearch,
});
</script>
