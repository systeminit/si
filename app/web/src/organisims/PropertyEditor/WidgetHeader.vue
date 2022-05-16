<template>
  <div
    class="flex flex-row w-full pl-7 pt-1 pb-1 mt-2 text-white cursor-pointer bg-gray-800 items-center h-12"
    @click="setCollapsed"
    v-show="isShown"
  >
    <div class="text-base font-extrabold">
      {{ name }}
    </div>
    <div
      v-for="(part, index) of path"
      :key="index"
      class="flex flex-row text-sm"
    >
      <ChevronLeftIcon class="h-5" /> {{ part }}
    </div>
    <div class="flex flex-grow justify-end pr-4">
      <SiButtonIcon tooltip-text="Expand" v-if="isCollapsed">
        <ChevronUpIcon />
      </SiButtonIcon>
      <SiButtonIcon tooltip-text="Collapse" v-else>
        <ChevronDownIcon />
      </SiButtonIcon>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { ChevronLeftIcon } from "@heroicons/vue/outline";
import { ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/solid";
import _ from "lodash";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";

const props = defineProps<{
  name: string;
  path: string[];
  collapsedPaths: Array<Array<string>>;
}>();
const emits = defineEmits<{
  (e: "toggle-collapsed", path: Array<string>): void;
}>();

const setCollapsed = () => {
  emits("toggle-collapsed", [props.name, ...props.path]);
};

const { name, path, collapsedPaths } = toRefs(props);
const { isShown, isCollapsed } = usePropertyEditorIsShown(
  name,
  path,
  collapsedPaths,
);
</script>
