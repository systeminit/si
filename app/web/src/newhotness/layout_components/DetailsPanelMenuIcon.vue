<template>
  <IconButton
    icon="dots-vertical"
    iconIdleTone="neutral"
    :selected="selected"
    :disabled="disabled"
  />
</template>

<script lang="ts" setup>
import { IconButton } from "@si/vue-lib/design-system";

defineProps({
  selected: { type: Boolean },
  disabled: { type: Boolean, default: false },
});
</script>
