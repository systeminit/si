<template>
  <ul>
    <li
      v-for="component in components"
      :key="component.componentName"
      :class="
        clsx(
          'ml-xs',
          'flex flex-row items-center gap-xs [&>*]:text-sm [&>*]:font-bold',
          themeClasses('[&>*]:border-neutral-400', '[&>*]:border-neutral-600'),
        )
      "
    >
      <TextPill mono :class="clsx('min-w-0', themeClasses('text-newhotness-greenlight', 'text-newhotness-greendark'))">
        <TruncateWithTooltip>
          {{ component.schemaName }}
        </TruncateWithTooltip>
      </TextPill>
      <TextPill
        mono
        :class="clsx('min-w-0', themeClasses('text-newhotness-purplelight', 'text-newhotness-purpledark'))"
      >
        <TruncateWithTooltip>
          {{ component.componentName }}
        </TruncateWithTooltip>
      </TextPill>
    </li>
  </ul>
</template>

<script setup lang="ts">
import { themeClasses, TextPill, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  components: Array<{ componentName: string; schemaName: string }>;
}>();
</script>
