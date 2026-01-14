<template>
  <ul
    :class="
      clsx(
        'flex flex-col gap-xs rounded border [&>li]:px-xs py-xs',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
  >
    <li
      v-if="edges.length > 0"
      :class="clsx('text-sm font-bold border-b', themeClasses('border-neutral-400', 'border-neutral-600'))"
    >
      {{ titleText }}
    </li>

    <ManagementConnectionInput
      v-if="selectComponent && parentComponentId"
      :existingEdges="edges"
      :parentComponentName="parentComponentName ?? 'this component'"
      :parentComponentId="parentComponentId"
    />
    <template v-if="edges.length > 0">
      <ManagementConnectionCard v-for="edge in edges" :key="edge.key" :componentId="edge.componentId" />
    </template>
  </ul>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import ManagementConnectionCard from "./ManagementConnectionCard.vue";
import ManagementConnectionInput from "./ManagementConnectionInput.vue";

defineProps({
  edges: { type: Array as PropType<SimpleConnection[]>, required: true },
  titleText: { type: String, default: "Management Connections" },
  selectComponent: { type: Boolean },
  parentComponentName: { type: String },
  parentComponentId: { type: String },
});
</script>
