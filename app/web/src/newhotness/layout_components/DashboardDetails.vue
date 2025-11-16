<template>
  <ul :class="['flex flex-row', gap, text]">
    <li
      :class="[
        padding,
        gap,
        'flex flex-row items-center',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      ]"
    >
      {{ components }} Components
    </li>
    <li
      :class="[
        padding,
        gap,
        'flex flex-row items-center',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      ]"
    >
      <StatusIndicatorIcon
        v-tooltip="'Resource'"
        type="resource"
        size="sm"
        status="exists"
      />
      {{ resources }} Resources
    </li>
    <li
      :class="[
        padding,
        gap,
        'flex flex-row items-center',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      ]"
    >
      <Icon
        name="tilde-circle"
        :class="themeClasses('text-warning-500', 'text-warning-300')"
        size="sm"
      />
      {{ diff }} Changes
    </li>
    <li
      :class="[
        padding,
        gap,
        'flex flex-row items-center',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      ]"
    >
      <StatusIndicatorIcon type="qualification" size="sm" status="failure" />
      {{ failed }} Failed Qualifications
    </li>
  </ul>
</template>

<script lang="ts" setup>
import { themeClasses, Icon } from "@si/vue-lib/design-system";
import { computed } from "vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

export type ChangeSetDetails = {
  components: number;
  resources: number;
  diff: number;
  failed: number;
};

export type TextSizes = "xs" | "sm" | "md";

const props = defineProps<{
  details: ChangeSetDetails | undefined;
  size?: TextSizes;
}>();

const diff = computed(() => props.details?.diff ?? 0);
const resources = computed(() => props.details?.resources ?? 0);
const failed = computed(() => props.details?.failed ?? 0);
const components = computed(() => props.details?.components ?? 0);

const text = computed(() => `text-${props.size || "sm"}`);
const padding = computed(() => (props.size !== "xs" ? "p-xs" : "p-2xs"));
const gap = computed(() => (props.size !== "xs" ? "gap-xs" : "gap-2xs"));
</script>
