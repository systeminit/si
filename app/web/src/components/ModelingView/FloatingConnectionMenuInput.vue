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
      <input
        class="flex-1 border-none outline-none bg-shade-100"
        ref="inputRef"
        :value="modelValue"
      />
      <div
        v-if="focused"
        class="flex flex-row flex-none gap-2xs items-center text-2xs"
      >
        <TextPill noVerticalPadding>Up</TextPill>
        <TextPill noVerticalPadding>Down</TextPill>
        <div>to navigate</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { clsx } from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { ref } from "vue";
import TextPill from "../TextPill.vue";

const inputRef = ref<InstanceType<typeof HTMLInputElement>>();

defineProps({
  modelValue: { type: String },
  focused: { type: Boolean },
  active: { type: Boolean },
});

const focus = () => {
  inputRef.value?.focus();
};

defineExpose({ focus });
</script>
