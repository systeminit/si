<template>
  <SiCollapsible
    :label="`${kind} &quot;${name}&quot;`"
    as="li"
    content-as="ul"
    default-open
    class="w-full"
    text-size="md"
    show-label-and-slot
    :force-theme="forceTheme"
    :hide-bottom-border="hideBottomBorderWhileClosed"
  >
    <template #label>
      <HealthIcon :status="status" size="md" hide-text />
    </template>
    <div
      class="px-4 pt-2 pb-4 w-full max-h-96 overflow-hidden"
      :class="borderClasses"
    >
      <CodeViewer :code="output" border :force-theme="forceTheme">
        <template #title><HealthIcon :status="status" /></template>
      </CodeViewer>
    </div>
  </SiCollapsible>
</template>

<script lang="ts" setup>
import { PropType, computed } from "vue";
import { ThemeValue } from "@/observable/theme";
import HealthIcon, { Health } from "@/molecules/HealthIcon.vue";
import SiCollapsible from "../SiCollapsible.vue";
import CodeViewer from "../CodeViewer.vue";

const props = defineProps({
  name: { type: String, required: true },
  kind: { type: String, required: true },
  output: { type: String, required: true },
  status: { type: String as PropType<Health>, required: true },
  forceTheme: { type: String as PropType<ThemeValue> },
  hideBottomBorderWhileClosed: { type: Boolean, default: false },
});

const borderClasses = computed(() => {
  let classes = "";

  if (props.hideBottomBorderWhileClosed) classes = "border-t";

  if (props.forceTheme === "dark")
    return `${classes} border-b border-neutral-600`;
  else if (props.forceTheme === "light")
    return `${classes}border-b border-neutral-200`;
  return `${classes}border-b border-neutral-200 dark:border-neutral-600`;
});
</script>
