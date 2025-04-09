<template>
  <div
    :class="
      clsx(
        'p-xs border-l-4 border relative',
        themeClasses('bg-shade-0', 'bg-shade-100'),
        statusColors,
      )
    "
  >
    <div class="flex gap-xs items-center">
      <Stack spacing="2xs" class="min-w-0">
        <div class="flex flex-row gap-2xs items-center">
          <Icon
            :name="outputSocket ? 'output-socket' : 'input-socket'"
            size="xs"
          />
          <div class="font-bold break-all line-clamp-4 pb-[1px] text-sm">
            {{ socket.def.label }}
          </div>
        </div>
        <div class="text-xs italic capsize">
          <TruncateWithTooltip class="pr-xs">
            {{ socket.parent.def.displayName }}
          </TruncateWithTooltip>
        </div>
      </Stack>

      <!-- ICONS AFTER THIS POINT ARE RIGHT ALIGNED DUE TO THE ml-auto STYLE ON THIS DIV -->
      <div class="ml-auto"></div>

      <!-- change status icon -->
      <div
        v-if="
          'changeStatus' in socket.def &&
          socket.def.changeStatus !== 'unmodified'
        "
        v-tooltip="{
          content: socket.def.changeStatus,
          theme: 'instant-show',
        }"
        class="cursor-pointer rounded hover:scale-125"
      >
        <StatusIndicatorIcon
          type="change"
          :status="(socket.def.changeStatus as string)"
          @click="viewsStore.setComponentDetailsTab('diff')"
        />
      </div>

      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import {
  Icon,
  Stack,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, PropType } from "vue";
import { tw } from "@si/vue-lib";
import { useViewsStore } from "@/store/views.store";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { DiagramSocketData } from "./ModelingDiagram/diagram_types";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const props = defineProps({
  socket: { type: Object as PropType<DiagramSocketData>, required: true },
  changeStatus: { type: String as PropType<ChangeStatus> },
  outputSocket: { type: Boolean },
});

const viewsStore = useViewsStore();

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
