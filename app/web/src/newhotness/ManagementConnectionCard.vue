<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs [&>*]:text-sm [&>*]:font-bold cursor-pointer mx-xs py-2xs',
        themeClasses('[&>*]:border-neutral-400', '[&>*]:border-neutral-600'),
        themeClasses('hover:bg-neutral-100', 'hover:bg-neutral-700'),
        selectable && [
          'border border-transparent p-2xs',
          themeClasses('hover:border-action-500', 'hover:border-action-300'),
          selected && themeClasses('bg-action-200', 'bg-action-900'),
        ],
      )
    "
    @click="selectable ? emit('select') : navigateToComponent($event)"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
  >
    <TextPill
      mono
      :class="
        clsx(
          'min-w-0',
          themeClasses('text-newhotness-greenlight bg-neutral-100', 'text-newhotness-greendark bg-neutral-900'),
        )
      "
    >
      <TruncateWithTooltip>
        {{ ctx.componentDetails.value[componentId]?.schemaVariantName ?? "unknown" }}
      </TruncateWithTooltip>
    </TextPill>
    <TextPill
      mono
      :class="
        clsx(
          'min-w-0 flex flex-row items-center gap-xs',
          themeClasses('text-newhotness-purplelight bg-neutral-100', 'text-newhotness-purpledark bg-neutral-900'),
        )
      "
    >
      <TruncateWithTooltip>
        {{ ctx.componentDetails.value[componentId]?.name ?? componentId }}
      </TruncateWithTooltip>
    </TextPill>
  </li>
</template>

<script setup lang="ts">
import { inject, ref } from "vue";
import clsx from "clsx";
import { TextPill, themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { assertIsDefined, Context } from "./types";

const props = defineProps({
  componentId: { type: String, required: true },
  selectable: { type: Boolean },
  selected: { type: Boolean },
});

const ctx = inject<Context>("CONTEXT");
assertIsDefined<Context>(ctx);

const emit = defineEmits<{
  (e: "select"): void;
}>();

const router = useRouter();
const isHovering = ref(false);

const navigateToComponent = (e: MouseEvent) => {
  e.stopPropagation();
  const params = { ...router.currentRoute.value.params };
  const query = { ...router.currentRoute.value.query };
  params.componentId = props.componentId;
  router.push({
    name: "new-hotness-component",
    params,
    query,
  });
};
</script>
