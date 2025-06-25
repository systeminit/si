<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs [&>*]:text-sm [&>*]:font-bold',
        themeClasses('[&>*]:border-neutral-400', '[&>*]:border-neutral-600'),
      )
    "
  >
    <TextPill mono class="text-purple">
      {{
        ctx.componentDetails.value[edge.componentId]?.name ?? edge.componentId
      }}
    </TextPill>
    <TextPill
      mono
      :class="themeClasses('text-green-light-mode', 'text-green-dark-mode')"
    >
      <!-- TODO(WENDY) - schema variant name goes here -->
      {{
        ctx.componentDetails.value[edge.componentId]?.schemaVariantName ??
        "unknown"
      }}
    </TextPill>
  </li>
</template>

<script setup lang="ts">
import { inject, PropType } from "vue";
import clsx from "clsx";
import { TextPill, themeClasses } from "@si/vue-lib/design-system";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import { assertIsDefined, Context } from "./types";

defineProps({
  edge: { type: Object as PropType<SimpleConnection>, required: true },
});

const ctx = inject<Context>("CONTEXT");
assertIsDefined<Context>(ctx);
</script>
