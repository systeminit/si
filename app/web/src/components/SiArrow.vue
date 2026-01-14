<template>
  <Icon name="chevron--down" size="xs" :class="props.nudge ? 'translate-y-0.5' : ''" />
</template>

<script setup lang="ts">
import { Icon } from "@si/vue-lib/design-system";

const props = defineProps<{
  nudge?: boolean;
}>();
</script>
