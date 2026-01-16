<template>
  <div :class="clsx('p-xs border-l-4 border relative', themeClasses('bg-shade-0', 'bg-shade-100'), statusColors)">
    <div class="flex gap-xs items-center">
      <Stack class="min-w-0" spacing="2xs">
        <div class="flex flex-row gap-2xs items-center">
          <Icon :name="statusIcon" size="xs" />
          <div class="font-bold break-all line-clamp-4 pb-[1px] text-sm">
            {{ subjectLabel }}
          </div>
        </div>
        <div class="text-xs italic capsize">
          <TruncateWithTooltip class="pr-xs">
            {{ componentName }}
          </TruncateWithTooltip>
        </div>
      </Stack>

      <!-- ICONS AFTER THIS POINT ARE RIGHT ALIGNED DUE TO THE ml-auto STYLE ON THIS DIV -->
      <div class="ml-auto"></div>

      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { Icon, Stack, themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import { computed, PropType } from "vue";
import { tw } from "@si/vue-lib";
import { ChangeStatus } from "@/api/sdf/dal/change_set";

const props = defineProps({
  subjectLabel: { type: String, required: true },
  componentName: { type: String, required: true },
  changeStatus: { type: String as PropType<ChangeStatus> },
  type: {
    type: String as PropType<"input-socket" | "output-socket" | "prop">,
    required: true,
  },
});

const statusIcon = computed(() => {
  switch (props.type) {
    case "input-socket":
      return "input-socket";
    case "output-socket":
      return "output-socket";
    case "prop":
    default:
      return "cursor";
  }
});

const statusColors = computed(() => {
  const unmodified = themeClasses(tw`border-shade-100`, tw`border-shade-0`);
  if (!props.changeStatus) return unmodified;
  const colors = {
    added: themeClasses(tw`border-success-500`, tw`border-success-400`),
    deleted: tw`border-destructive-500`,
    modified: tw`border-warning-400`,
    unmodified,
  };
  return colors[props.changeStatus];
});
</script>
