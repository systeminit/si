<template>
  <div class="flex flex-col gap-2xs">
    <div
      class="bg-neutral-900 rounded-sm border border-neutral-600 flex flex-row p-xs items-center text-sm gap-2"
    >
      <Icon
        v-if="expandable"
        :name="expanded ? 'chevron-down' : 'chevron-right'"
        class="cursor-pointer text-neutral-500 hover:bg-neutral-600 active:bg-neutral-700"
        @click="expanded = !expanded"
      />
      <span
        class="grow text-sm overflow-hidden text-ellipsis whitespace-nowrap"
        >{{ text }}</span
      >
      <Icon
        name="copy"
        size="sm"
        class="cursor-pointer hover:bg-neutral-600 active:bg-neutral-700"
        @click="copyText"
      />
    </div>
    <div
      v-if="expandable && expanded"
      class="bg-neutral-900 rounded-sm border border-neutral-600 p-xs text-sm leading-4 break-all"
    >
      {{ text }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { ref } from "vue";

const props = defineProps<{
  text: string;
  expandable?: boolean;
}>();

const expanded = ref(false);

const copyText = () => {
  navigator.clipboard.writeText(props.text);
  emit("copied");
};

const emit = defineEmits<{
  (e: "copied"): void;
}>();
</script>
