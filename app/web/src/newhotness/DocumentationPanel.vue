<template>
  <CollapsingFlexItem :open="open" @toggle="emit('toggle')">
    <template #header> Documentation </template>
    <div
      v-if="
        !docs &&
        !component.schemaVariantDocLink &&
        !component.schemaVariantDescription
      "
      class="flex flex-col items-center p-sm gap-sm"
    >
      <div
        :class="
          clsx(
            'p-sm rounded-full',
            themeClasses('bg-neutral-100', 'bg-neutral-900'),
          )
        "
      >
        <Icon name="docs" />
      </div>
      <div>No documentation available</div>
    </div>
    <template v-if="!docs">
      <p v-if="component.schemaVariantDocLink">
        <a
          :href="component.schemaVariantDocLink"
          target="_blank"
          tabindex="-1"
          >{{ component.schemaVariantName }}</a
        >
      </p>
      <p>
        <VueMarkdown :source="component.schemaVariantDescription ?? ''" />
      </p>
    </template>
    <template v-else>
      <VButton
        class="border-0 mr-2em"
        icon="arrow--left"
        label="Back"
        size="sm"
        tone="shade"
        variant="ghost"
        @click="emit('cleardocs')"
      />
      <p v-if="docLink">
        <a :href="docLink" target="_blank">{{ component.schemaVariantName }}</a>
      </p>
      <p>{{ docs }}</p>
    </template>
  </CollapsingFlexItem>
</template>

<script lang="ts" setup>
import { Icon, themeClasses, VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";
import VueMarkdown from "vue-markdown-render";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";

defineProps({
  component: { type: Object as PropType<BifrostComponent>, required: true },
  docs: { type: String },
  docLink: { type: String },
  open: { type: Boolean },
});

const emit = defineEmits<{
  (e: "cleardocs"): void;
  (e: "toggle"): void;
}>();
</script>
