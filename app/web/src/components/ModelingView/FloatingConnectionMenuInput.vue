<template>
  <div
    :class="
      clsx(
        'border-t-2 border-x-2 pt-xs px-xs pb-xs',
        active && themeClasses('bg-neutral-100', 'bg-neutral-800'),
      )
    "
  >
    <div
      :class="
        clsx(
          'border flex flex-row gap-xs px-xs h-[34px]',
          active
            ? themeClasses(
                'border-action-500 bg-shade-0',
                'border-action-300 bg-shade-100',
              )
            : themeClasses('border-neutral-400', 'border-neutral-600'),
          disabled && 'cursor-not-allowed',
        )
      "
    >
      <VormInput
        ref="inputRef"
        v-model="searchString"
        :class="clsx('flex-1 border-none outline-none')"
        noStyles
        :disabled="disabled"
      />
      <div
        v-if="active"
        class="flex flex-row flex-none gap-3xs items-center text-2xs"
      >
        <TextPill tighter variant="key">Up</TextPill>
        <TextPill tighter variant="key">Down</TextPill>
        <div class="leading-snug">to navigate</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { clsx } from "clsx";
import { themeClasses, VormInput, TextPill } from "@si/vue-lib/design-system";
import { ref } from "vue";

const inputRef = ref<InstanceType<typeof VormInput>>();

defineProps({
  active: { type: Boolean },
  disabled: { type: Boolean },
});

const searchString = ref("");

const focus = () => {
  inputRef.value?.focus();
};

defineExpose({ focus, searchString });
</script>
