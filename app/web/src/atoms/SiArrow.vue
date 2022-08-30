<template>
  <Icon name="chevron--down" :class="props.nudge ? 'translate-y-0.5' : ''" />
</template>

<script setup lang="ts">
import Icon from "@/ui-lib/Icon.vue";

const props = defineProps<{
  nudge?: boolean;
}>();
</script>
