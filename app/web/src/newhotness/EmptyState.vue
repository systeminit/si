<template>
  <div class="flex flex-col items-center gap-sm text-center">
    <div
      :class="
        clsx(
          !iconNoBg && [!noTopMargin && 'mt-sm', 'p-sm rounded-full', themeClasses('bg-neutral-200', 'bg-neutral-700')],
        )
      "
    >
      <Icon :name="icon" :size="iconSize" />
    </div>
    <!-- Primary text should be a concise title -->
    <div :class="themeClasses('text-neutral-800', 'text-neutral-200')">
      <slot>
        {{ text }}
      </slot>
    </div>
    <!-- Secondary text can be a longer explanation and/or a link -->
    <slot name="secondary">
      <div
        v-if="secondaryText"
        :class="
          clsx(
            secondaryClickable && [
              'cursor-pointer hover:underline',
              themeClasses('hover:text-action-500', 'hover:text-action-300'),
            ],
            themeClasses('text-neutral-600', 'text-neutral-400'),
          )
        "
        @click="secondaryClick"
      >
        {{ secondaryText }}
      </div>
    </slot>
    <!-- Third row of text only allowed as a link-->
    <slot name="final">
      <div
        v-if="finalLinkText"
        :class="
          clsx(
            'cursor-pointer hover:underline',
            themeClasses('hover:text-action-500 text-neutral-600', 'hover:text-action-300 text-neutral-400'),
          )
        "
        @click="emit('final')"
      >
        {{ finalLinkText }}
      </div>
    </slot>
  </div>
</template>

<script setup lang="ts">
import { Icon, IconNames, IconSizes, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";

const props = defineProps({
  text: { type: String, required: true },
  secondaryText: { type: String },
  finalLinkText: { type: String },
  secondaryClickable: { type: Boolean },
  icon: { type: String as PropType<IconNames>, required: true },
  iconSize: { type: String as PropType<IconSizes> },
  iconNoBg: { type: Boolean },
  noTopMargin: { type: Boolean },
});

const emit = defineEmits<{
  (e: "secondary"): void;
  (e: "final"): void;
}>();

const secondaryClick = () => {
  if (props.secondaryClickable) {
    emit("secondary");
  }
};
</script>
