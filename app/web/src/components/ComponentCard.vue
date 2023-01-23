<template>
  <div
    :class="
      clsx(
        'rounded-md p-xs',
        componentColorIsDark ? 'text-shade-0' : 'text-shade-100',
      )
    "
    :style="{ backgroundColor: component.color }"
  >
    <Inline align-y="center">
      <Icon :name="component.icon" size="lg" class="shrink-0" />
      <Stack spacing="xs" class="">
        <div class="font-bold">
          {{ component.displayName }}
        </div>
        <div class="text-xs italic capsize">
          <div class="truncate pr-xs">{{ component.schemaName }}</div>
        </div>
      </Stack>
    </Inline>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import Inline from "@/ui-lib/layout/Inline.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import Stack from "@/ui-lib/layout/Stack.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const componentsStore = useComponentsStore();
const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

const componentColor = tinycolor(component.value.color || "#FFF");
const componentColorIsDark = componentColor.isDark();
</script>
