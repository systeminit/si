<template>
  <div
    class="flex nowrap gap-3 p-3 dark:border-neutral-600 border-b-2 justify-between"
  >
    <label
      class="relative text-neutral-400 focus-within:text-gray-600 block flex-grow"
    >
      <input
        v-model="search"
        :placeholder="props.placeholder"
        class="w-full text-black px-1 py-[0.4375rem] pl-2.5 text-sm rounded-sm border dark:text-black bg-neutral-100 border-neutral-200 placeholder:italic placeholder:text-neutral-400"
      />
    </label>
    <button
      v-if="!props.autosearch"
      class="w-[2rem] text-action-"
      @click="performSearch"
    >
      <SearchIcon class="w-full text-neutral-500" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { SearchIcon } from "@heroicons/vue/solid";
import { ref, watch } from "vue";

let search = ref<string>("");

const props = withDefaults(
  defineProps<{
    placeholder?: string;
    autosearch?: boolean;
  }>(),
  {
    placeholder: "search",
    autosearch: false,
  },
);

const emits = defineEmits<{
  (e: "search", v: string): void;
}>();

watch(
  () => search.value,
  (search) => props.autosearch && emits("search", search),
);

const performSearch = () => emits("search", search.value);
</script>
