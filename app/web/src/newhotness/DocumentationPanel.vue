<template>
  <CollapsingFlexItem :open="open" @toggle="emit('toggle')">
    <template #header> Documentation </template>
    <div class="flex flex-col items-center p-xs gap-sm [&>p]:w-full">
      <EmptyState
        v-if="
          !docs &&
          !component.schemaVariantDocLink &&
          !component.schemaVariantDescription
        "
        text="No documentation available"
        icon="docs"
      />
      <template v-if="!docs">
        <p v-if="component.schemaVariantDocLink">
          <a
            :href="component.schemaVariantDocLink"
            target="_blank"
            tabindex="-1"
          >
            {{ component.schemaVariantName }}
          </a>
        </p>
        <MarkdownRender :source="component.schemaVariantDescription ?? ''" />
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
          <a :href="docLink" target="_blank">{{
            component.schemaVariantName
          }}</a>
        </p>
        <MarkdownRender :source="docs" />
      </template>
    </div>
  </CollapsingFlexItem>
</template>

<script lang="ts" setup>
import { VButton } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import MarkdownRender from "./MarkdownRender.vue";
import EmptyState from "./EmptyState.vue";

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
