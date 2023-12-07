<template>
  <div
    v-if="component"
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
        <div
          ref="componentNameRef"
          v-tooltip="componentNameTooltip"
          class="font-bold break-all line-clamp-4 pb-[2px]"
        >
          {{ component.displayName }}
        </div>
        <div class="text-xs italic capsize">
          <div class="truncate pr-xs">{{ component.schemaName }}</div>
        </div>
      </Stack>

      <!-- change status icon -->
      <div class="ml-auto cursor-pointer">
        <StatusIndicatorIcon
          type="change"
          :status="component.changeStatus"
          @click="componentsStore.setComponentDetailsTab('diff')"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import { useTheme, Stack, Icon } from "@si/vue-lib/design-system";
import {
  ComponentId,
  FullComponent,
  useComponentsStore,
} from "@/store/components.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const { theme } = useTheme();

const componentsStore = useComponentsStore();
const component = computed(
  (): FullComponent | undefined =>
    componentsStore.componentsById[props.componentId],
);

const primaryColor = tinycolor(component.value?.color ?? "000000");

// body bg
const bodyBg = computed(() => {
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  return tinycolor(bodyBgHsl);
});

const componentNameRef = ref();
const componentNameTooltip = computed(() => {
  if (
    componentNameRef.value &&
    componentNameRef.value.scrollHeight > componentNameRef.value.offsetHeight
  ) {
    return {
      content: componentNameRef.value.textContent,
      delay: { show: 700, hide: 10 },
    };
  } else {
    return {};
  }
});
</script>
