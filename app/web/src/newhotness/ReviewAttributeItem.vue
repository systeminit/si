<template>
  <div
    :class="
      clsx(
        'flex flex-col gap-xs p-xs border',
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-800',
        ),
        '[&>*]:h-10 [&>div]:p-xs [&>h1]:py-xs',
      )
    "
  >
    <h1>{{ path }}</h1>
    <div
      v-if="diff.new"
      :class="
        clsx(
          'flex flex-row items-center gap-xs',
          themeClasses('bg-success-200', 'bg-success-900'),
        )
      "
    >
      <div
        :class="
          clsx(
            'text-xl flex-none',
            themeClasses('text-success-600', 'text-success-500'),
          )
        "
      >
        +
      </div>
      <TruncateWithTooltip class="py-2xs"
        >{{ diff.new.$value }} SOURCE:
        {{ diff.new.$source }}</TruncateWithTooltip
      >
    </div>
    <div
      v-if="diff.old"
      :class="
        clsx(
          'flex flex-row items-center gap-xs',
          themeClasses('text-neutral-600', 'text-neutral-400'),
        )
      "
    >
      <div class="text-xl flex-none">-</div>
      <TruncateWithTooltip class="line-through mr-auto py-2xs"
        >{{ diff.old.$value }} SOURCE:
        {{ diff.old.$source }}</TruncateWithTooltip
      >
      <IconButton
        class="flex-none"
        icon="undo"
        iconTone="shade"
        iconIdleTone="shade"
        tooltip="Revert to old value"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";
import { AttributeDiff } from "@/workers/types/entity_kind_types";
import { AttributePath } from "@/api/sdf/dal/component";

defineProps({
  path: { type: String as PropType<AttributePath>, required: true },
  diff: { type: Object as PropType<AttributeDiff>, required: true },
});
</script>
