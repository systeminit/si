<template>
  <Icon
    :name="open ? 'chevron-down' : 'chevron-right'"
    size="sm"
    :class="['flex-none', !noPointer && 'cursor-pointer']"
  />
</template>

<script setup lang="ts">
import Icon from "../icons/Icon.vue";

defineProps<{
  open: boolean;
  noPointer?: boolean;
}>();
</script>
