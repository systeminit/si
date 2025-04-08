<template>
  <div
    :class="
      clsx(
        'border-t-2 border-x-2 p-sm pb-xs',
        active && themeClasses('bg-neutral-100', 'bg-neutral-800'),
      )
    "
  >
    <div
      :class="
        clsx(
          'border flex flex-row gap-xs px-xs h-[34px]',
          themeClasses('bg-shade-0', 'bg-shade-100'),
          focused && themeClasses('border-action-500', 'border-action-300'),
        )
      "
    >
      <VormInput
        ref="inputRef"
        v-model="searchString"
        :class="
          clsx(
            'flex-1 border-none outline-none',
            themeClasses('bg-shade-0', 'bg-shade-100'),
          )
        "
        noStyles
      />
      <div
        v-if="focused"
        class="flex flex-row flex-none gap-3xs items-center text-2xs"
      >
        <TextPill tighter>Up</TextPill>
        <TextPill tighter>Down</TextPill>
        <div class="leading-snug">to navigate</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { clsx } from "clsx";
import { themeClasses, VormInput } from "@si/vue-lib/design-system";
import { ref } from "vue";
import TextPill from "../TextPill.vue";

const inputRef = ref<InstanceType<typeof VormInput>>();

defineProps({
  focused: { type: Boolean },
  active: { type: Boolean },
});

const searchString = ref("");

const focus = () => {
  inputRef.value?.focus();
};

defineExpose({ focus, searchString });
</script>
