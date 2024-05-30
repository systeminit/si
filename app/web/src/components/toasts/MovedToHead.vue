<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon :name="icon" :class="iconClass" />
    <div class="flex flex-col text-sm text-center">
      <div>
        The change set
        <span class="font-bold italic">{{ changeSetName }}</span> has been
        {{ action }}.
      </div>
      <div>You are now on HEAD.</div>
    </div>
    <VButton
      label="Ok"
      size="sm"
      variant="solid"
      tone="empty"
      class="self-stretch text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
    />
  </div>
</template>

<script lang="ts" setup>
import { Icon, IconNames, VButton } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";

const props = defineProps({
  icon: { type: String as PropType<IconNames>, default: "none" },
  action: { type: String, required: true },
  changeSetName: { type: String, required: true },
});

const iconClass = computed(() => {
  if (props.icon === "trash") return "text-destructive-500";
  else if (props.icon === "tools") return "text-success-500";
  else return "";
});
</script>
