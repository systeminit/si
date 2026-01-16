<template>
  <div
    :class="
      clsx(
        'flex text-neutral-500 dark:text-neutral-400 border-b items-center px-xs py-2xs gap-xs h-[33px] select-none',
        iconIsButton && 'cursor-pointer',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
    @click="emit('click')"
    @mouseleave="onEndHover"
    @mouseover="onHover"
  >
    <div class="flex-none empty:hidden">
      <slot name="icon">
        <IconButton
          v-if="icon && iconIsButton"
          :icon="icon"
          iconIdleTone="neutral"
          :hovered="hover"
          :selected="selected"
          @click="emit('click')"
        />
        <Icon v-else-if="icon" :name="icon" />
      </slot>
    </div>

    <TruncateWithTooltip
      :class="clsx('grow font-bold', variant === 'title' ? 'uppercase text-md leading-6' : 'text-sm break-words')"
    >
      <slot name="label">{{ label }}</slot>
    </TruncateWithTooltip>
    <div class="flex-none empty:hidden">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, IconButton, IconNames, themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, ref } from "vue";

export type SidebarSubpanelTitleVariant = "title" | "subtitle";

const props = defineProps({
  label: { type: String },
  icon: { type: String as PropType<IconNames> },
  iconIsButton: { type: Boolean },
  selected: { type: Boolean },
  variant: {
    type: String as PropType<SidebarSubpanelTitleVariant>,
    default: "title",
  },
});

const emit = defineEmits(["click"]);

const hover = ref(false);

const onHover = () => {
  hover.value = true;
};
const onEndHover = () => {
  hover.value = false;
};
</script>
