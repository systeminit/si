<template>
  <div
    ref="nodeRef"
    :class="
      clsx(
        'tree-node',
        internalScrolling && 'overflow-hidden flex flex-col',
        internalScrolling && isOpen ? 'flex-1' : 'flex-none',
        styleAsGutter && enableGroupToggle && ['ml-sm border-l', themeClasses('border-neutral-100', 'border-neutral-700')],
      )
    "
  >
    <div
      :class="
        clsx(
          'relative cursor-pointer group flex-none',
          !noIndentationOrLeftBorder && !styleAsGutter &&
            {
              none: '',
              '2xs': 'border-l-[1px]',
              xs: 'border-l-2',
              sm: 'border-l-[3px]',
              md: 'border-l-4',
              lg: 'border-l-[6px]',
              xl: 'border-l-8',
              '2xl': 'border-l-[12px]',
            }[leftBorderSize],
          isHover &&
            'dark:outline-action-300 outline-action-500 outline z-10 -outline-offset-1 outline-1',
          isSelected && themeClasses('bg-action-100', 'bg-action-900'),
          showSelection && isSelected
            ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300 py-0'
            : (!styleAsGutter && 'dark:hover:text-action-300 hover:text-action-500'),
          classes,
          (enableDefaultHoverClasses || styleAsGutter) &&
            'bg-neutral-100 dark:bg-neutral-700 group/tree',
        )
      "
      :style="{
        borderLeftColor: color,
        // backgroundColor: bodyBg,
      }"
      @click="tryOpen(clickLabelToToggle)"
    >
      <Icon
        v-if="styleAsGutter && enableGroupToggle"
        :name="isOpen ? 'chevron--down' : 'chevron--right'"
        class="absolute left-[-21px] translate-y-1/2 hover:text-action-300 group-hover/tree:text-action-500 dark:group-hover/tree:text-action-300"
        />
      <div
        :class="
          clsx(
            !styleAsGutter && 'flex flex-row items-center px-xs w-full gap-1',
            labelClasses,
            enableDefaultHoverClasses &&
              'font-bold select-none hover:text-action-500 dark:hover:text-action-300',
          )
        "
      >
        <Icon
          v-if="enableGroupToggle && alwaysShowArrow && !styleAsGutter"
          :name="isOpen ? 'chevron--down' : 'chevron--right'"
          size="lg"
          @click.stop="tryOpen()"
        />
        <div
          v-if="slots.primaryIcon"
          :class="
            clsx(
              'flex-none',
              primaryIconClasses,
              enableGroupToggle &&
                !alwaysShowArrow &&
                'group-hover:scale-0 transition-all',
            )
          "
        >
          <slot name="primaryIcon" />
        </div>
        <Icon
          v-else-if="primaryIcon"
          :name="primaryIcon"
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

        <div :class="clsx(!styleAsGutter && 'flex flex-col select-none overflow-hidden py-2xs w-full')">
          <slot v-if="useDifferentLabelWhenOpen && isOpen" name="openLabel">
            {{ openLabel }}
          </slot>
          <slot v-else name="label">{{ label }}</slot>
        </div>
        <!-- group open/close controls -->
        <div
          v-if="enableGroupToggle && !alwaysShowArrow && !styleAsGutter"
          class="absolute left-[0px] cursor-pointer"
          @click.stop="tryOpen()"
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
          !noIndentationOrLeftBorder && !styleAsGutter &&
            {
              none: '',
              '2xs': 'pl-2xs',
              xs: 'pl-xs',
              sm: 'pl-sm',
              md: 'pl-md',
              lg: 'pl-lg',
              xl: 'pl-xl',
              '2xl': 'pl-2xl',
            }[indentationSize],
          internalScrolling && 'overflow-auto flex-1',
          childrenContainerClasses,
        )
      "
    >
      <slot />
    </div>
    <div :class="staticContentClasses">
      <slot name="staticContent" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref, useSlots } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon, IconNames, SpacingSizes, themeClasses } from "..";

const props = defineProps({
  // The label can be set via a prop or via a slot
  label: { type: String },

  // Turn this on if this TreeNode has children inside and can be toggled open/closed
  enableGroupToggle: { type: Boolean },

  // All TreeNodes start open by default, set this to false if you want it to start closed
  defaultOpen: { type: Boolean, default: true },

  // Set this to false if you only want the TreeNode to be toggleable via the arrow
  clickLabelToToggle: { type: Boolean, default: true },

  // This determines the color of the left border, not used if leftBorderSize is none
  color: { type: String, default: "#000" },

  // The primary icon on the left side, will merge with the arrow unless alwaysShowArrow is enabled
  primaryIcon: { type: String as PropType<IconNames> },

  // Prevents the arrow from the default minizing behavior
  alwaysShowArrow: { type: Boolean },

  // External hooks for showing hover and selection of this TreeNode
  isSelected: { type: Boolean },
  showSelection: { type: Boolean },
  isHover: { type: Boolean },

  // Props to adjust the internal indentation for the TreeNode's children and the thickness of the left border
  indentationSize: {
    type: String as PropType<SpacingSizes>,
    default: "xs",
  },
  leftBorderSize: {
    type: String as PropType<SpacingSizes>,
    default: "sm",
  },
  noIndentationOrLeftBorder: { type: Boolean },

  // these props allow you to use a different label when open, there is also an openLabel slot as well
  useDifferentLabelWhenOpen: { type: Boolean },
  openLabel: { type: String },

  // turn this on if you want to have a list of TreeNodes with internal scrolling on each
  internalScrolling: { type: Boolean },

  // some default hover styling that can also apply group hover styling to the label, icons, or children of this TreeNode
  enableDefaultHoverClasses: { type: Boolean },

  // an alternative style for TabGroup that is not intended to nest and does not use many of the adjustment props
  styleAsGutter: { type: Boolean },

  // direct class injection into various spots in this component - try to use sparingly!
  classes: { type: String },
  labelClasses: { type: String },
  childrenContainerClasses: { type: String },
  staticContentClasses: { type: String },
  primaryIconClasses: { type: String, default: "mr-xs" },
});

const nodeRef = ref<HTMLElement>();
const isOpen = ref(props.defaultOpen);
const slots = useSlots();

const tryOpen = (enabled = true) => {
  if (!enabled) return;
  else toggleIsOpen();
};

const toggleIsOpen = (state = !isOpen.value) => {
  isOpen.value = state;
};

defineExpose({ isOpen, toggleIsOpen });
</script>
