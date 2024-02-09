<template>
  <!-- NEW SEARCH BAR WITH FILTERS -->
  <div
    v-if="featureFlagsStore.SEARCH_FILTERS"
    class="flex nowrap gap-xs p-xs dark:border-neutral-600 border-b justify-between"
  >
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
    </label>
    <button
      class="w-6 text-action- text-neutral-500 hover:text-shade-100 dark:hover:text-shade-0 hover:scale-110"
      @click="triggerSearch"
    >
      <Icon name="search" />
    </button>
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
import { ref, watch } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();

const emit = defineEmits<{
  (e: "search", searchTerm: string): void;
  (e: "update:modelValue", newValue: string): void;
}>();

const props = defineProps({
  placeholder: { type: String, default: "search" },
  modelValue: { type: String },
  autoSearch: { type: Boolean },
});

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
</script>
