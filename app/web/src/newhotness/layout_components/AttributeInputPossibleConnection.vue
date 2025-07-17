<template>
  <div
    v-if="connection"
    :class="
      clsx(
        `absolute top-0 left-0 w-full h-[${virtualItemSize}px]`,
        'possible-connections grid gap-xs cursor-pointer border border-transparent',
        'px-xs py-2xs',
        isConnectionSelected && [
          'input-selected-item',
          themeClasses('bg-action-200', 'bg-action-900'),
        ],
        themeClasses(
          'hover:border-action-500 active:active:bg-action-200',
          'hover:border-action-300 active:active:bg-action-900',
        ),
      )
    "
    :style="{
      transform: `translateY(${virtualItemStart}px)`,
    }"
    @click.left="emit('selectConnection', virtualItemIndex)"
  >
    <div
      class="flex flex-row items-center gap-xs font-mono [&>*]:basis-1/2 [&>*]:flex-grow [&>*]:max-w-fit"
    >
      <TruncateWithTooltip
        :class="themeClasses('text-green-light-mode', 'text-green-dark-mode')"
      >
        {{ connection.schemaName }}
      </TruncateWithTooltip>
      <TruncateWithTooltip class="text-purple">
        {{ connection.componentName }}
      </TruncateWithTooltip>
    </div>
    <div class="flex flex-row gap-2xs items-center">
      <template v-for="(item, itemIndex) in connection.pathArray" :key="item">
        <TruncateWithTooltip
          class="flex-1 max-w-fit"
          :style="`flex-basis: ${100 / (connection.pathArray.length ?? 0)}%`"
        >
          {{ item }}
        </TruncateWithTooltip>
        <div v-if="itemIndex !== (connection.pathArray.length ?? 0) - 1">/</div>
      </template>
    </div>
    <TruncateWithTooltip
      :class="
        clsx(
          'font-mono',
          connection.value === null && 'italic text-neutral-400',
        )
      "
    >
      <template
        v-if="
          connection.kind === 'array' ||
          connection.kind === 'map' ||
          connection.kind === 'object' ||
          connection.kind === 'json'
        "
      >
        {{ connection.kind }}
      </template>
      <template v-else-if="connection.value === null"> No value yet </template>
      <template v-else>
        {{ connection.value }}
      </template>
    </TruncateWithTooltip>
  </div>
</template>

<script setup lang="ts">
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { UIPotentialConnection } from "./AttributeInput.vue";

defineProps<{
  connection?: UIPotentialConnection;
  isConnectionSelected: boolean;
  virtualItemSize: number;
  virtualItemIndex: number;
  virtualItemStart: number;
}>();

const emit = defineEmits<{
  (e: "selectConnection", index: number): void;
}>();
</script>

<style lang="css" scoped>
.possible-connections.grid {
  grid-template-columns: minmax(0, 40%) minmax(0, 40%) minmax(0, 20%);
}
</style>
