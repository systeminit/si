<template>
  <div
    class="bg-neutral-900 rounded-sm border border-neutral-600 flex flex-row p-xs items-center text-sm gap-3"
  >
    <span class="grow text-sm leading-4 break-all">{{ text }}</span>
    <Icon
      name="clipboard-copy"
      size="xs"
      class="cursor-pointer"
      @click="copyText"
    />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";

const props = defineProps<{
  text: string;
}>();

const copyText = () => {
  navigator.clipboard.writeText(props.text);
};
</script>
