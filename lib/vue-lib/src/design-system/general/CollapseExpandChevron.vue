<template>
  <Icon
    :name="open ? 'chevron-down' : 'chevron-right'"
    size="sm"
    class="cursor-pointer flex-none"
  />
</template>

<script setup lang="ts">
import Icon from "../icons/Icon.vue";

defineProps({
  open: Boolean,
});
</script>
