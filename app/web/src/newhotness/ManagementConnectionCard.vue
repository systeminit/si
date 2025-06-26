<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs [&>*]:text-sm [&>*]:font-bold',
        themeClasses('[&>*]:border-neutral-400', '[&>*]:border-neutral-600'),
        selectable && [
          'border border-transparent p-2xs cursor-pointer',
          themeClasses('hover:border-action-500', 'hover:border-action-300'),
          selected && themeClasses('bg-action-200', 'bg-action-900'),
        ],
      )
    "
    @click="emit('select')"
  >
    <TextPill mono class="text-purple min-w-0">
      <TruncateWithTooltip>{{
        ctx.componentDetails.value[componentId]?.name ?? componentId
      }}</TruncateWithTooltip>
    </TextPill>
    <TextPill
      mono
      :class="
        clsx(
          'min-w-0',
          themeClasses('text-green-light-mode', 'text-green-dark-mode'),
        )
      "
    >
      <TruncateWithTooltip>
        {{
          ctx.componentDetails.value[componentId]?.schemaVariantName ??
          "unknown"
        }}
      </TruncateWithTooltip>
    </TextPill>
  </li>
</template>

<script setup lang="ts">
import { inject } from "vue";
import clsx from "clsx";
import {
  TextPill,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { assertIsDefined, Context } from "./types";

defineProps({
  componentId: { type: String, required: true },
  selectable: { type: Boolean },
  selected: { type: Boolean },
});

const ctx = inject<Context>("CONTEXT");
assertIsDefined<Context>(ctx);

const emit = defineEmits<{
  (e: "select"): void;
}>();
</script>
