<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon :name="icon" :class="iconClass" />
    <div class="flex flex-col text-sm text-center">
      <div>
        The change set
        <span class="font-bold italic">{{ changeSetName }}</span> has been {{ action }}.
      </div>
      <div>You are now on HEAD.</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, IconNames, getToneTextColorClass } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";

const props = defineProps({
  icon: { type: String as PropType<IconNames>, default: "none" },
  action: { type: String, required: true },
  changeSetName: { type: String, required: true },
});

const iconClass = computed(() => {
  if (props.icon === "trash") return getToneTextColorClass("destructive");
  else if (props.icon === "tools") return getToneTextColorClass("success");
  else return "";
});
</script>
