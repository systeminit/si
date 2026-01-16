<template>
  <img :src="svgUrl" />
</template>

<script lang="ts" setup>
import { useTheme, ThemeValue } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";
import NoAssetsLight from "@/assets/images/no-assets__light.svg?url";
import NoAssetsDark from "@/assets/images/no-assets__dark.svg?url";
import ActionsLight from "@/assets/images/actions__light.svg?url";
import ActionsDark from "@/assets/images/actions__dark.svg?url";
import ApplyChangesLight from "@/assets/images/apply-changes__light.svg?url";
import ApplyChangesDark from "@/assets/images/apply-changes__dark.svg?url";
import ConnectEdgesLight from "@/assets/images/connect-edges__light.svg?url";
import ConnectEdgesDark from "@/assets/images/connect-edges__dark.svg?url";
import CustomizeDark from "@/assets/images/customize__dark.svg?url";
import CustomizeLight from "@/assets/images/customize__light.svg?url";
import NoComponentsLight from "@/assets/images/no-components__light.svg?url";
import NoComponentsDark from "@/assets/images/no-components__dark.svg?url";
import NoChangesDark from "@/assets/images/no-changes__dark.svg?url";
import FuncsDark from "@/assets/images/funcs__dark.svg?url";
import CustomizeBigDark from "@/assets/images/customize-big__dark.svg?url";

const { theme: appTheme } = useTheme();

const props = defineProps({
  name: { type: String as PropType<EMPTY_STATE_ICON_NAMES>, required: true },
  fixedTheme: { type: String as PropType<ThemeValue> },
});

const svgUrl = computed(() => {
  return BIG_ICONS[props.name][props.fixedTheme ?? appTheme.value] as EMPTY_STATE_ICON_NAMES;
});
</script>

<script lang="ts">
export const BIG_ICONS = Object.freeze({
  actions: { light: ActionsLight, dark: ActionsDark },
  "apply-changes": { light: ApplyChangesLight, dark: ApplyChangesDark },
  "connect-edges": { light: ConnectEdgesLight, dark: ConnectEdgesDark },
  customize: { light: CustomizeLight, dark: CustomizeDark },
  "no-assets": { light: NoAssetsLight, dark: NoAssetsDark },
  "no-components": { light: NoComponentsLight, dark: NoComponentsDark },
  "no-changes": { light: NoChangesDark, dark: NoChangesDark }, // currently there is only a dark version of this EmptyStateIcon
  funcs: { light: FuncsDark, dark: FuncsDark }, // currently there is only a dark version of this EmptyStateIcon
  "customize-big": { light: CustomizeBigDark, dark: CustomizeBigDark }, // currently there is only a dark version of this EmptyStateIcon
});

export type EMPTY_STATE_ICON_NAMES = keyof typeof BIG_ICONS;
</script>
