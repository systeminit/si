<template>
  <li
    :class="
      clsx(
        'rounded border p-xs',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
  >
    <!-- TODO(WENDY) - clean this up to display the data better -->
    {{ ctx.componentNames.value[edge.componentId] ?? edge.componentId }}
  </li>
</template>

<script setup lang="ts">
import { inject, PropType } from "vue";
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import { assertIsDefined, Context } from "./types";

defineProps({
  edge: { type: Object as PropType<SimpleConnection>, required: true },
});

const ctx = inject<Context>("CONTEXT");
assertIsDefined<Context>(ctx);
</script>
