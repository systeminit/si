<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center rounded-full border w-fit py-[1px] text-xs cursor-pointer',
        filter.iconName ? 'pr-2xs pl-3xs' : 'px-2xs',
        selected
          ? 'bg-action-500 dark:bg-action-400 border-action-500 dark:border-action-400 text-shade-0'
          : 'hover:text-action-400 hover:dark:text-action-300 hover:border-action-400 hover:dark:border-action-300 border-action-700 dark:border-action-200',
      )
    "
  >
    <Icon
      v-if="filter.iconName"
      :name="filter.iconName"
      size="sm"
      :class="clsx(iconClasses, extraPadding && 'mx-2xs')"
      :style="filter.iconColor && !selected ? { color: filter.iconColor } : {}"
    />
    <div :class="clsx(filter.iconName && 'pr-2xs')">
      <slot name="label">{{ filterLabel }}</slot>
    </div>
    <div v-if="filter.count || filter.count === 0" class="font-bold">
      {{ filter.count }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { PropType, computed } from "vue";
import Icon from "../icons/Icon.vue";
import { getToneTextColorClass } from "../utils/color_utils";
import { Filter } from "./SiSearch.vue";

const MAX_FILTER_LABEL_LENGTH = 40;

const props = defineProps({
  filter: { type: Object as PropType<Filter>, required: true },
  selected: { type: Boolean },
});

const iconClasses = computed(() => {
  if (props.filter.iconTone && !props.selected) {
    return getToneTextColorClass(props.filter.iconTone);
  } else return "";
});

// Add an icon to this list if it needs to have additional X padding to look right
const extraPaddingIcons = ["cloud-upload", "code-deployed"];

const extraPadding = computed(
  () =>
    props.filter.iconName &&
    (props.filter.iconName.includes("logo") ||
      extraPaddingIcons.includes(props.filter.iconName)),
);

const filterLabel = computed(() => {
  if (props.filter.name.length < MAX_FILTER_LABEL_LENGTH) {
    return props.filter.name;
  } else {
    return `${props.filter.name.substring(0, MAX_FILTER_LABEL_LENGTH)}...`;
  }
});
</script>
