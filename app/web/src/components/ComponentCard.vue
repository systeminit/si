<template>
  <div
    :class="
      clsx(
        'p-xs border-l-4 border relative',
        !titleCard && 'rounded-md',
        'toDelete' in component.def && component.def.toDelete && 'opacity-70',
        'fromBaseChangeSet' in component.def &&
          component.def.fromBaseChangeSet &&
          'opacity-70',
      )
    "
    :style="{
      borderColor: component.def.color,
      backgroundColor: `#${bodyBg.toHex()}`,
    }"
  >
    <div class="flex gap-2xs items-center">
      <Icon
        v-if="component.def.componentType !== ComponentType.View"
        :name="component.def.icon"
        size="lg"
        class="shrink-0"
      />
      <Icon v-else name="logo-si" size="lg" class="shrink-0" />
      <Icon
        :name="COMPONENT_TYPE_ICONS[component.def.componentType]"
        size="lg"
        class="shrink-0"
      />
      <Stack spacing="xs" class="min-w-0">
        <div
          ref="componentNameRef"
          v-tooltip="componentNameTooltip"
          class="font-bold break-all line-clamp-4 pb-[1px]"
        >
          {{ displayName }}
        </div>
        <div class="text-xs italic capsize">
          <TruncateWithTooltip class="pr-xs">
            {{ component.def.schemaName }}
          </TruncateWithTooltip>
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
          v-if="'canBeUpgraded' in component.def && component.def.canBeUpgraded"
          type="upgradable"
          @click="upgradeComponent"
        />
      </div>

      <!-- change status icon -->
      <div
        v-if="
          'changeStatus' in component.def &&
          component.def.changeStatus !== 'unmodified'
        "
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
          @click="viewStore.setComponentDetailsTab('diff')"
        />
      </div>

      <!-- Slot for additional icons/buttons -->
      <IconButton
        v-if="
          component.def.schemaDocsLink && component.def.schemaDocsLink !== ''
        "
        icon="bookOpen"
        class="shrink-0"
        @click="navigateToSchemaDocs(component.def.schemaDocsLink)"
      />
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
  IconButton,
  Stack,
  TruncateWithTooltip,
  COMPONENT_TYPE_ICONS,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramViewData,
} from "./ModelingDiagram/diagram_types";

const props = defineProps<{
  titleCard?: boolean;
  component: DiagramGroupData | DiagramNodeData | DiagramViewData;
}>();

const { theme } = useTheme();

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();

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

const displayName = computed(
  () =>
    ("displayName" in props.component.def && props.component.def.displayName) ||
    ("name" in props.component.def && props.component.def.name) ||
    "Asset",
);

function navigateToSchemaDocs(docsLink: string): void {
  window.open(docsLink, "_blank");
}

const upgradeComponent = async () => {
  viewStore.setSelectedComponentId(null);
};
</script>
