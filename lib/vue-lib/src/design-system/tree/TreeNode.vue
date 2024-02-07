<template>
  <div ref="nodeRef" class="tree-node">
    <div
      :class="
        clsx(
          'relative cursor-pointer group',
          {
            none: '',
            '2xs': 'border-l-[1px]',
            xs: 'border-l-2',
            sm: 'border-l-[3px]',
            md: 'border-l-4',
            lg: 'border-l-[6px]',
            xl: 'border-l-8',
          }[leftBorderSize],
          isHover &&
            'dark:outline-action-300 outline-action-500 outline z-10 -outline-offset-1',
          isSelected && themeClasses('bg-action-100', 'bg-action-900'),
          classes,
        )
      "
      :style="{
        borderLeftColor: color,
        // backgroundColor: bodyBg,
      }"
      @click="toggleOpen(clickLabelToToggle)"
    >
      <div
        :class="
          clsx('flex flex-row items-center px-xs w-full gap-1', labelClasses)
        "
      >
        <Icon
          v-if="enableGroupToggle && alwaysShowArrow"
          :name="isOpen ? 'chevron--down' : 'chevron--right'"
          size="lg"
        />
        <Icon
          v-if="icon"
          :name="icon"
          size="md"
          :class="
            clsx(
              'mr-xs flex-none',
              enableGroupToggle &&
                !alwaysShowArrow &&
                'group-hover:scale-0 transition-all',
            )
          "
        />

        <div class="flex flex-col select-none overflow-hidden py-xs">
          <slot name="label">{{ label }}</slot>
        </div>
        <!-- group open/close controls -->
        <div
          v-if="enableGroupToggle && !alwaysShowArrow"
          class="absolute left-[0px] cursor-pointer"
          @click.stop="toggleOpen()"
        >
          <Icon
            :name="isOpen ? 'chevron--down' : 'chevron--right'"
            size="lg"
            class="scale-[40%] translate-x-[-9px] translate-y-[13px] group-hover:scale-100 group-hover:translate-x-0 group-hover:translate-y-0 transition-all"
          />
        </div>

        <div class="ml-auto flex flex-none">
          <slot name="icons" />
        </div>
      </div>
    </div>
    <!-- children -->
    <div
      v-if="enableGroupToggle && isOpen"
      :class="
        clsx(
          {
            none: '',
            '2xs': 'pl-2xs',
            xs: 'pl-xs',
            sm: 'pl-sm',
            md: 'pl-md',
            lg: 'pl-lg',
            xl: 'pl-xl',
          }[indentationSize],
        )
      "
    >
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon, IconNames, themeClasses } from "..";

defineProps({
  label: { type: String },
  clickLabelToToggle: { type: Boolean },
  color: { type: String, default: "#000" },
  icon: { type: String as PropType<IconNames> },
  isSelected: { type: Boolean },
  isHover: { type: Boolean },
  enableGroupToggle: { type: Boolean },
  classes: { type: String },
  labelClasses: { type: String },
  indentationSize: {
    type: String as PropType<"none" | "2xs" | "xs" | "sm" | "md" | "lg" | "xl">,
    default: "xs",
  },
  leftBorderSize: {
    type: String as PropType<"none" | "2xs" | "xs" | "sm" | "md" | "lg" | "xl">,
    default: "sm",
  },
  alwaysShowArrow: { type: Boolean },
});

const nodeRef = ref<HTMLElement>();
const isOpen = ref(true);

const toggleOpen = (enabled = true) => {
  if (!enabled) return;
  else isOpen.value = !isOpen.value;
};
</script>
