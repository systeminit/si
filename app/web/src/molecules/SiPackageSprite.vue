<template>
  <RouterLink
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :to="{
      name: 'workspace-lab-packages',
      params: { ...route.params, packageSlug: slug },
    }"
  >
    <Icon :name="icon || 'cat'" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ props.name }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { useRoute } from "vue-router";
import Icon from "@/ui-lib/icons/Icon.vue";
import { IconNames } from "@/ui-lib/icons/icon_set";

const props = defineProps({
  name: { type: String },
  icon: { type: String as PropType<IconNames> },
  slug: { type: String },
});

const route = useRoute();
</script>
