<template>
  <div
    :class="
      clsx(
        'border p-xs',
        kind === 'error'
          ? themeClasses(
              'bg-destructive-100 border-destructive-600',
              'bg-newhotness-destructive border-destructive-400',
            )
          : themeClasses('border-neutral-600', 'border-neutral-600'),
      )
    "
  >
    <!-- First row for icon and destructive text -->
    <div
      :class="
        clsx(
          'flex flex-row items-center gap-xs',
          kind === 'error' &&
            themeClasses('text-destructive-600', 'text-destructive-200'),
        )
      "
    >
      <Icon
        v-if="iconName"
        :name="iconName"
        :class="kind === 'success' && 'text-success-200'"
        size="xs"
      />
      <span class="text-sm">{{ text }}</span>

      <!-- Slot for anything, but mainly for button(s) (do things like toggle the subtitle) -->
      <div class="ml-auto">
        <slot name="right"></slot>
      </div>
    </div>

    <!-- Second row for a subtitle in neutral text -->
    <div
      v-if="showSubtitle && subtitle"
      :class="
        clsx(
          'flex flex-row items-center mt-xs text-sm',
          themeClasses('text-neutral-700', 'text-neutral-400'),
        )
      "
    >
      <span>{{ subtitle }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";

const props = defineProps<{
  kind: "loading" | "error" | "neutral" | "success";
  text: string;
  subtitle?: string;
  showSubtitle?: boolean;
}>();

const iconName = computed(() => {
  if (props.kind === "loading") return "loader";
  if (props.kind === "error") return "x-circle";
  if (props.kind === "success") return "check-circle";
  return undefined;
});
</script>
