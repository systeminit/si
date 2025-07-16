<template>
  <CollapsingFlexItem :open="open" @toggle="emit('toggle')">
    <template #header> Documentation </template>
    <div class="flex flex-col items-center p-xs gap-xs [&>.prose]:w-full">
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
        <div v-if="component.schemaVariantDocLink" class="w-full">
          <a
            :href="component.schemaVariantDocLink"
            target="_blank"
            tabindex="-1"
            :class="
              clsx(
                'no-underline hover:underline font-bold text-lg',
                themeClasses('text-action-500', 'text-action-300'),
              )
            "
          >
            {{ component.schemaVariantName }}
          </a>
        </div>
        <MarkdownRender
          :source="component.schemaVariantDescription ?? ''"
          disableMaxWidth
        />
      </template>
      <template v-else>
        <div class="flex flex-row items-center gap-xs w-full">
          <div v-if="docLink">
            <a
              :href="docLink"
              target="_blank"
              :class="
                clsx(
                  'no-underline hover:underline font-bold text-lg',
                  themeClasses('text-action-500', 'text-action-300'),
                )
              "
            >
              {{ component.schemaVariantName }}
            </a>
          </div>
          <IconButton
            class="ml-auto"
            icon="x"
            tooltip="Close"
            tooltipPlacement="top"
            size="sm"
            iconTone="shade"
            @click="emit('cleardocs')"
          />
        </div>
        <MarkdownRender :source="docs" />
      </template>
    </div>
  </CollapsingFlexItem>
</template>

<script lang="ts" setup>
import { themeClasses, IconButton } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
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
