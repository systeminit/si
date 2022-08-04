<template>
  <VButton
    size="xs"
    button-rank="tertiary"
    button-type="neutral"
    :label="label"
    :icon-style="iconStyle"
    :icon="icon"
    :tool-tip="toolTip"
    :disabled="disabled"
  />
</template>

<script setup lang="ts">
import VButton, { ButtonIconStyle } from "@/molecules/VButton.vue";
import _, { IconName } from "@/molecules/VButton/Icon.vue";

withDefaults(
  defineProps<{
    label: string;
    toolTip?: string;
    disabled?: boolean;
    iconStyle?: ButtonIconStyle;
    icon?: IconName;
  }>(),
  {
    toolTip: undefined,
    iconStyle: "none",
    icon: undefined,
  },
);
</script>
