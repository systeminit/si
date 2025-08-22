<template>
  <div
    v-if="displayKind !== 'hidden'"
    :class="
      clsx(
        'flex flex-row items-center gap-xs font-mono h-10 p-xs',
        old
          ? themeClasses('text-neutral-600', 'text-neutral-400')
          : themeClasses('bg-success-200', 'bg-success-900'),
      )
    "
  >
    <div
      :class="
        clsx(
          'text-xl flex-none w-sm text-center',
          !old && themeClasses('text-success-600', 'text-success-500'),
        )
      "
    >
      {{ old ? "-" : "+" }}
    </div>
    <!-- <div class="text-2xs">
      {{ rawValue }} / {{ rawSource }}
    </div> -->
    <TruncateWithTooltip
      v-if="displayKind === 'value' || displayKind === 'complex'"
      :class="clsx('py-2xs', old && 'line-through')"
    >
      {{ rawValue }}
      <span v-if="displayKind === 'complex'">
        <span v-if="rawSource.fromSchema">(default)</span>
        <span v-else>(complex source data: {{ rawSource }})</span>
      </span>
    </TruncateWithTooltip>
    <AttributeValueBox v-else-if="displayKind === 'subscription'">
      <div
        :class="
          clsx(
            'max-w-full flex flex-row items-center px-2xs [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit [&>*]:py-2xs',
            old && 'line-through',
          )
        "
      >
        <TruncateWithTooltip :class="clsx(!old && 'text-purple')">
          {{ rawSource.componentName }}
        </TruncateWithTooltip>
        <div class="flex-none">/</div>
        <TruncateWithTooltip
          :class="themeClasses('text-neutral-600', 'text-neutral-400')"
        >
          {{ rawValue }}
        </TruncateWithTooltip>
      </div>
    </AttributeValueBox>

    <div v-if="revertible" class="mr-auto" />
    <IconButton
      v-if="revertible"
      class="flex-none"
      icon="undo"
      iconTone="shade"
      iconIdleTone="shade"
      tooltip="Revert to old value"
      @click="emit('revert')"
    />
  </div>
</template>

<script setup lang="ts">
import {
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { AttributeSourceAndValue } from "@/workers/types/entity_kind_types";
import AttributeValueBox from "./layout_components/AttributeValueBox.vue";

type DisplayKind = "hidden" | "value" | "subscription" | "complex";

const props = defineProps<{
  sourceAndValue: AttributeSourceAndValue;
  old?: boolean;
  revertible?: boolean;
}>();

const rawValue = computed(() => props.sourceAndValue.$value);
const rawSource = computed(() => props.sourceAndValue.$source);

const displayKind = computed<DisplayKind>(() => {
  if ("component" in rawSource.value) return "subscription";
  else if ("value" in rawSource.value) return "value";
  // TODO(Wendy) - complex AV diffs?
  return "hidden";
});

const emit = defineEmits<{
  (e: "revert"): void;
}>();
</script>
