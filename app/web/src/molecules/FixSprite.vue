<template>
  <SiCollapsible
    as="li"
    class="w-full"
    content-as="ul"
    :default-open="false"
    hide-bottom-border
  >
    <template #label>
      <div
        class="flex flex-row items-center gap-2.5 text-sm relative"
        :class="classes"
      >
        <VormInput type="checkbox" no-label />
        <Icon name="tools" size="xl" class="text-destructive-500" />
        <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
          {{ fix.name }}
        </div>
      </div>
    </template>
    <template #default>
      <div class="pl-8 pr-2 py-4">
        <span class="font-bold">Recommendation: </span>{{ fix.recommendation }}
      </div>
    </template>
  </SiCollapsible>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Icon from "@/ui-lib/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";

type Fix = {
  id: number;
  name: string;
  recommendation: string;
};

const props = defineProps<{
  fix: Fix;
  class?: string;
}>();

const classes = computed(() => props.class);
</script>
