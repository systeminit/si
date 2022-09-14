<template>
  <SiCollapsible
    :label="`${kind} &quot;${name}&quot;`"
    :default-open="!defaultClosed"
    text-size="md"
    show-label-and-slot
    :force-theme="forceTheme"
    :hide-bottom-border="hideBottomBorderWhileClosed"
  >
    <div
      class="w-full text-sm p-xs whitespace-nowrap overflow-hidden text-ellipsis"
    >
      Component: LINK TO COMPONENT HERE
    </div>
    <template #label>
      <HealthIcon :status="status" size="md" hide-text />
    </template>
    <div
      class="px-xs pb-xs max-h-96 overflow-hidden flex"
      :class="borderClasses"
    >
      <div class="flex-grow">
        <CodeViewer :code="output" border :force-theme="forceTheme">
          <template #title><HealthIcon :status="status" /></template>
        </CodeViewer>
      </div>
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
  defaultClosed: { type: Boolean, default: false },
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
