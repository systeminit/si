<template>
  <div
    v-show="showHeader"
    class="flex flex-row w-full pl-7 pt-1 pb-1 mt-2 cursor-pointer items-center h-12"
    @click="setCollapsed"
  >
    <div
      v-for="(part, index) of displayPath"
      :key="index"
      class="flex flex-row items-center"
    >
      <span v-if="index === 0" class="text-base font-bold">
        {{ part }}
      </span>
      <span v-else class="text-sm">
        <ChevronLeftIcon class="pl-2 h-5 inline" /> {{ part }}
      </span>
    </div>
    <div class="flex flex-grow justify-end pr-4">
      <SiButtonIcon v-if="isCollapsed" ignore-text-color tooltip-text="Expand">
        <ChevronUpIcon />
      </SiButtonIcon>
      <SiButtonIcon v-else ignore-text-color tooltip-text="Collapse">
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
import { PropertyPath } from "@/api/sdf/dal/property_editor";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
}>();
const emits = defineEmits<{
  (e: "toggle-collapsed", path: Array<string>): void;
}>();

const setCollapsed = () => {
  if (props.path) {
    console.log("collapsing", { triggerPath: props.path.triggerPath });
    emits("toggle-collapsed", props.path.triggerPath);
  }
};

const { name, path, collapsedPaths } = toRefs(props);

const displayPath = computed(() => {
  if (path && path.value) {
    // Always chop off the root path
    return path.value.displayPath.slice(0, path.value.displayPath.length - 1);
  } else {
    return [];
  }
});

const { isShown, isCollapsed } = usePropertyEditorIsShown(
  name,
  collapsedPaths,
  path,
  true,
);

const showHeader = computed(() => {
  if (!displayPath.value.length) {
    return false;
  } else {
    return isShown.value;
  }
});
</script>
