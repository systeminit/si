<template>
  <div
    :class="
      clsx(
        'p-xs border-l-4 border relative',
        titleCard ? 'mb-xs' : 'rounded-md',
        component.def.toDelete && 'opacity-70',
        component.def.fromBaseChangeSet && 'opacity-70',
      )
    "
    :style="{
      borderColor: component.def.color,
      backgroundColor: `#${bodyBg.toHex()}`,
    }"
  >
    <div class="flex gap-2xs items-center">
      <Icon :name="component.def.icon" size="lg" class="shrink-0" />
      <Icon
        :name="COMPONENT_TYPE_ICONS[component.def.componentType]"
        size="lg"
        class="shrink-0"
      />
      <Stack spacing="xs" class="">
        <div
          ref="componentNameRef"
          v-tooltip="componentNameTooltip"
          class="font-bold break-all line-clamp-4 pb-[1px]"
        >
          {{ component.def.displayName }}
        </div>
        <div class="text-xs italic capsize">
          <div class="truncate pr-xs">{{ component.def.schemaName }}</div>
        </div>
      </Stack>

      <!-- ICONS AFTER THIS POINT ARE RIGHT ALIGNED DUE TO THE ml-auto STYLE ON THIS DIV -->
      <div
        v-tooltip="{
          content: 'Upgrade',
          theme: 'instant-show',
        }"
        class="ml-auto cursor-pointer rounded hover:scale-125"
      >
        <StatusIndicatorIcon
          v-if="component.def.canBeUpgraded"
          type="upgradable"
          @click="upgradeComponent"
        />
      </div>

      <!-- change status icon -->
      <div
        v-if="component.def.changeStatus !== 'unmodified'"
        v-tooltip="{
          content:
            component.def.changeStatus.charAt(0).toUpperCase() +
            component.def.changeStatus.slice(1),
          theme: 'instant-show',
        }"
        class="cursor-pointer rounded hover:scale-125"
      >
        <StatusIndicatorIcon
          type="change"
          :status="component.def.changeStatus"
          @click="componentsStore.setComponentDetailsTab('diff')"
        />
      </div>

      <!-- Slot for additional icons/buttons -->
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import {
  useTheme,
  Icon,
  Stack,
  COMPONENT_TYPE_ICONS,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";

const props = defineProps<{
  titleCard: boolean;
  component: DiagramGroupData | DiagramNodeData;
}>();

const { theme } = useTheme();

const componentsStore = useComponentsStore();

const primaryColor = tinycolor(props.component.def.color);

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

const upgradeComponent = async () => {
  componentsStore.setSelectedComponentId(null);
  await componentsStore.UPGRADE_COMPONENT(
    props.component.def.id,
    props.component.def.displayName,
  );
};
</script>
