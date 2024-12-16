<template>
  <div class="flex flex-row gap-sm items-center mb-md">
    <div class="flex flex-col gap-2xs grow">
      <div
        ref="titleDiv"
        v-tooltip="titleTooltip"
        class="text-lg font-bold line-clamp-3 break-words"
      >
        {{ title }}
      </div>
      <div>{{ subtitle }}</div>
    </div>
    <slot />
    <RouterLink :to="{ name: 'workspaces' }">
      <VButton label="Return To Workspaces" tone="neutral" />
    </RouterLink>
  </div>
</template>

<script lang="ts" setup>
import { VButton } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";

const props = defineProps<{
  title: string;
  subtitle: string;
}>();

const titleDiv = ref<HTMLElement>();
const titleTooltip = computed(() => {
  if (
    titleDiv.value &&
    titleDiv.value.scrollHeight > titleDiv.value.offsetHeight
  ) {
    return {
      content: props.title,
      delay: { show: 700, hide: 10 },
    };
  } else {
    return {};
  }
});
</script>
