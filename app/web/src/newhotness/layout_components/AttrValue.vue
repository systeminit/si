<template>
  <div
    :class="
      clsx(
        'max-w-full flex flex-row items-center [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit',
        'p-2xs text-sm font-bold',
        strikeout && 'line-through',
        isSecret &&
          themeClasses('text-green-light-mode', 'text-green-dark-mode'),
      )
    "
  >
    <TruncateWithTooltip v-if="showComponentName" class="text-purple">
      {{ componentName }}
    </TruncateWithTooltip>
    <div v-if="showComponentName" class="flex-none">/</div>
    <TruncateWithTooltip
      :class="themeClasses('text-neutral-600', 'text-neutral-400')"
    >
      <template v-if="!isSecret && value !== 'null'">
        {{ value }}
      </template>
      <template v-else> &lt; {{ path }} &gt; </template>
    </TruncateWithTooltip>
  </div>
</template>

<script setup lang="ts">
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";

const showComponentName = computed(() => props.componentName !== "undefined");

const props = defineProps<{
  isSecret: boolean;
  componentName: string | undefined;
  path: string | undefined;
  value: string | undefined;
  strikeout?: boolean;
}>();
</script>
