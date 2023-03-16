<template>
  <div
    :class="
      clsx(
        'rounded-md p-xs border-l-4 border relative',
        component.changeStatus === 'deleted' && 'opacity-70',
      )
    "
    :style="{
      borderColor: component.color,
      backgroundColor: `#${bodyBg.toHex()}`,
    }"
  >
    <div class="flex gap-xs items-center">
      <Icon :name="component.icon" size="lg" class="shrink-0" />
      <Stack spacing="xs" class="">
        <div class="font-bold break-all line-clamp-4">
          {{ component.displayName }}
        </div>
        <div class="text-xs italic capsize">
          <div class="truncate pr-xs">{{ component.schemaName }}</div>
        </div>
      </Stack>

      <!-- change status icon -->
      <div class="ml-auto">
        <Icon
          v-if="component.changeStatus === 'added'"
          name="plus-circle"
          class="text-success-500"
        />
        <Icon
          v-if="component.changeStatus === 'deleted'"
          name="x"
          class="text-destructive-500"
        />
        <Icon
          v-if="component.changeStatus === 'modified'"
          name="tilde-circle"
          class="text-warning-500"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import { useTheme } from "@/ui-lib/theme_tools";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const { theme } = useTheme();

const componentsStore = useComponentsStore();
const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

const primaryColor = tinycolor(component.value.color);

// body bg
const bodyBg = computed(() => {
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  return tinycolor(bodyBgHsl);
});
</script>
