<template>
  <div
    class="hidden md:flex md:w-50 align-bottom md:flex-col md:fixed md:inset-y-16 min-h-screen"
    :class="placement"
  >
    <SiSidebarContent :placeholder="props.placeholder" />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import SiSidebarContent from "@/atoms/SiSidebarContent.vue";

const props = defineProps<{
  placeholder: string;
  placeRight?: boolean;
}>();

const placement = computed(() => {
  if (props.placeRight) {
    return "right-0";
  }
  return "left-0";
});
</script>
