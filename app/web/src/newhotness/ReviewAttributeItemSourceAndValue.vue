<template>
  <div
    v-if="displayKind !== 'hidden'"
    :class="
      clsx(
        'flex flex-row items-center gap-xs font-mono h-10 p-xs rounded-sm',
        old
          ? themeClasses('text-neutral-600', 'text-neutral-400')
          : themeClasses('bg-success-200', 'bg-newhotness-success'),
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

    <AttributeValueBox v-if="displayKind === 'subscription'" class="min-w-0">
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
          <template v-if="secret">
            <!-- TODO(Wendy) - Ideally we would put the secret's name here -->
            secret
          </template>
          <template v-if="rawValue !== undefined">
            {{ rawValue }}
          </template>
          <template v-else> &lt;{{ rawSource.path }}&gt; </template>
        </TruncateWithTooltip>
      </div>
    </AttributeValueBox>
    <TruncateWithTooltip
      v-else
      :class="clsx('py-2xs min-w-0', old && 'line-through')"
    >
      <template v-if="displayKind === 'secret'">
        <!-- TODO(Wendy) - Ideally we would put the secret's name here -->
        Secret {{ old ? "Removed" : "Added" }}
      </template>
      <template v-else>
        {{ rawValue }}
      </template>
    </TruncateWithTooltip>
  </div>
</template>

<script setup lang="ts">
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { AttributeSourceAndValue } from "@/workers/types/entity_kind_types";
import AttributeValueBox from "./layout_components/AttributeValueBox.vue";

type DisplayKind = "hidden" | "value" | "subscription" | "secret" | "complex";

const props = defineProps<{
  sourceAndValue: AttributeSourceAndValue;
  old?: boolean;
  secret?: boolean;
}>();

const rawValue = computed(() => props.sourceAndValue.$value);
const rawSource = computed(() => props.sourceAndValue.$source);

const displayKind = computed<DisplayKind>(() => {
  if ("component" in rawSource.value) return "subscription";
  if (props.secret) return "secret";
  if ("value" in rawSource.value) return "value";
  // TODO(Wendy) - complex AV diffs?
  return "hidden";
});

const emit = defineEmits<{
  (e: "revert"): void;
}>();
</script>
