<template>
  <div
    v-show="showHeader"
    :class="
      clsx(
        'flex flex-row w-full px-xs cursor-pointer items-center',
        nestingLevel === 0 ? 'pt-md' : 'pt-sm',
      )
    "
    @click="setCollapsed"
  >
    <div class="flex justify-start pr-xs">
      <SiButtonIcon
        ignoreTextColor
        :tooltipText="isCollapsed ? 'Expand' : 'Collapse'"
        :icon="isCollapsed ? 'chevron--right' : 'chevron--down'"
      />
    </div>

    <div class="">
      <div class="text-xs flex gap-xxs pb-2xs">
        <template v-for="parent in pathParents" :key="parent">
          <div>{{ parent }}</div>
          <Icon name="chevron--right" size="xs" class="last:hidden" />
        </template>
      </div>
      <div
        :class="
          clsx('font-bold  capsize', nestingLevel === 0 ? 'text-lg' : 'text-sm')
        "
      >
        {{ pathLast }}
      </div>

      <div
        v-for="(part, index) of displayPath"
        :key="index"
        class="flex flex-row items-center"
      >
        <!-- <span
          v-if="index === 0"
          :class="
            clsx(
              nestingLevel === 0 ? 'text-lg font-bold' : 'text-base font-bold',
            )
          "
        >
          {{ part }}
        </span>
        <span v-else class="text-xs pl-xs">
          <Icon name="chevron--left" size="sm" /> {{ part }}
        </span> -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon } from "@si/vue-lib/design-system";
import SiButtonIcon from "@/components/SiButtonIcon.vue";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import { PropertyPath } from "@/api/sdf/dal/property_editor";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
}>();
const emits = defineEmits<{
  (e: "toggle-collapsed", path: Array<string>): void;
}>();

const nestingLevel = computed(() =>
  props.path ? props.path.displayPath.length - 2 : 0,
);

const setCollapsed = () => {
  if (props.path) {
    emits("toggle-collapsed", props.path.triggerPath);
  }
};

const { name, path, collapsedPaths } = toRefs(props);

const displayPath = computed(() => {
  // Always chop off the root path
  return path?.value?.displayPath.slice(0, -1).reverse() || [];
});

const pathParents = computed(() => displayPath.value.slice(undefined, -1));
const pathLast = computed(() => displayPath.value.slice(-1)[0]);

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
