<template>
  <div
    :class="
      clsx(
        'border flex flex-row gap-xs px-xs',
        classes,
        themeClasses('bg-shade-0', 'bg-shade-100'),
        show ? activeClasses : inactiveClasses,
      )
    "
  >
    <slot name="left"> </slot>
    <slot
      :focus="() => (show = true)"
      :blur="() => (show = showInstructions ?? false)"
      :class="
        clsx(
          'flex-1 border-none outline-none [&_input]:placeholder:italic',
          themeClasses('bg-shade-0', 'bg-shade-100'),
        )
      "
    />
    <div
      v-if="show"
      class="flex flex-row flex-none gap-3xs items-center text-2xs"
    >
      <TextPill v-for="p in pills" :key="p" tighter>{{ p }}</TextPill>
      <span class="leading-snug">{{ instructions }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { clsx } from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import TextPill from "@/components/TextPill.vue";

const props = withDefaults(
  defineProps<{
    classes?: string;
    activeClasses: string;
    inactiveClasses?: string;
    showInstructions?: boolean;
    pills?: string[];
    instructions?: string;
  }>(),
  {
    classes: tw`py-xs`,
    inactiveClasses: "",
  },
);

const show = ref(props.showInstructions);
</script>
