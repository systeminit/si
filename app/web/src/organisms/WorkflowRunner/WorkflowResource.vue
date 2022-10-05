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
      Component:
      <span
        class="font-bold cursor-pointer text-action-500 dark:text-action-300 hover:dark:text-action-500 hover:text-action-300"
        @click="showConfirmationsFor(component.id)"
        >{{ component.name }}</span
      >
    </div>
    <template #label>
      <HealthIcon :health="status" size="md" hide-text />
    </template>
    <div
      class="px-xs pb-xs max-h-96 overflow-hidden flex"
      :class="borderClasses"
    >
      <div class="flex-grow">
        <CodeViewer :code="output" border :force-theme="forceTheme">
          <template #title><HealthIcon :health="status" /></template>
        </CodeViewer>
      </div>
    </div>
  </SiCollapsible>
</template>

<script lang="ts" setup>
import { PropType, computed } from "vue";
import { ThemeValue } from "@/observable/theme";
import HealthIcon from "@/molecules/HealthIcon.vue";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { useComponentsStore } from "@/store/components.store";
import SiCollapsible from "../SiCollapsible.vue";
import CodeViewer from "../CodeViewer.vue";
import { ComponentListItem } from "../StatusBar/StatusBarTabPanelComponentList.vue";

const props = defineProps({
  component: {
    type: Object as PropType<ComponentListItem>,
    required: true,
  },
  name: { type: String, required: true },
  kind: { type: String, required: true },
  output: { type: String, required: true },
  status: { type: String as PropType<ResourceHealth>, required: true },
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

const componentsStore = useComponentsStore();

const showConfirmationsFor = (componentId: number) => {
  componentsStore.setSelectedComponentId(componentId);
  // TODO(wendy) - code to open the StatusBar to the Confirmations panel
};
</script>
