<template>
  <TreeNode
    leftBorderSize="xl"
    :color="theme === 'dark' ? '#525252' : '#E5E5E5'"
    staticContentClasses="border-b border-neutral-200 dark:border-neutral-600"
  >
    <template #label>
      <div class="flex flex-row items-center text-sm overflow-hidden w-full">
        <StatusIndicatorIcon type="action-runner" :status="runner.status" />
        <div class="flex flex-col pl-xs flex-shrink overflow-hidden flex-grow">
          <div>{{ `${runner.displayName}` }}</div>
          <div
            ref="componentNameRef"
            v-tooltip="componentNameTooltip"
            :class="
              clsx(
                'truncate cursor-pointer',
                componentsStore.allComponentsById[runner.componentId]
                  ? 'dark:text-action-300 text-action-500 hover:underline font-bold'
                  : 'text-neutral-500 dark:text-neutral-400 line-through',
                isHover && 'underline',
              )
            "
            @click="onClick"
            @mouseenter="onHoverStart"
            @mouseleave="onHoverEnd"
          >
            {{ `${runner.componentName}` }}
          </div>
        </div>
      </div>
    </template>
    <template #icons>
      <div
        v-if="runner.resource"
        class="dark:text-action-300 text-action-500 flex-none cursor-pointer flex flex-row gap-xs"
      >
        <ActionRunnerDetails
          :health="runner.resource.status"
          :message="
            [
              `${formatTitle(runner.actionKind)} ${runner.schemaName}`,
              runner.resource.message ?? (runner.resource.status === 'ok' ? 'Completed successfully' : 'Error'),
            ].filter((f) => f.length > 0)
          "
        />
        <IconButton
          v-if="runner.resource?.payload"
          tooltip="show code"
          rotate="down"
          icon="code-pop"
          iconHover="code-pop-square"
          :selected="codeViewerShowing"
          @click="toggleCodeViewerShowing"
        />
      </div>
    </template>
    <template #staticContent>
      <div v-if="codeViewerShowing && runner.resource?.payload" class="relative w-full">
        <CodeViewer
          :code="JSON.stringify(runner.resource.payload, null, 2)"
          class="dark:text-neutral-50 text-neutral-900"
        >
          <template #title>
            <div class="font-bold">
              {{ runner.resource.message ?? "Resource Code" }}
              <ActionRunnerDetails
                :health="runner.resource.status"
                :message="
                  [`${formatTitle(runner.actionKind)} ${runner.schemaName}`, runner.resource.message ?? ''].filter(
                    (f) => f.length > 0,
                  )
                "
              />
            </div>
          </template>
        </CodeViewer>
      </div>
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { PropType, computed, ref } from "vue";
import { TreeNode, useTheme, IconButton } from "@si/vue-lib/design-system";
import { DeprecatedActionRunner } from "@/store/actions.store";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useViewsStore } from "@/store/views.store";
import CodeViewer from "../CodeViewer.vue";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import ActionRunnerDetails from "./ActionRunnerDetails.vue";

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();

const { theme } = useTheme();

const props = defineProps({
  runner: { type: Object as PropType<DeprecatedActionRunner>, required: true },
  hideTopBorder: { type: Boolean },
});

const formatTitle = (title: string) => {
  return title
    .split(" ")
    .map((t) => `${t[0]?.toUpperCase()}${t.slice(1).toLowerCase()}`)
    .join(" ");
};

const componentNameRef = ref();
const componentNameTooltip = computed(() => {
  if (componentNameRef.value) {
    if (!componentsStore.allComponentsById[props.runner.componentId]) {
      return {
        content: `Component "${componentNameRef.value.textContent}" does not exist ${
          changeSetsStore.headSelected ? "on head" : "in this change set"
        }.`,
        delay: { show: 700, hide: 10 },
      };
    } else if (componentNameRef.value.scrollWidth > componentNameRef.value.offsetWidth) {
      return {
        content: componentNameRef.value.textContent,
        delay: { show: 700, hide: 10 },
      };
    }
  }
  return {};
});

const viewStore = useViewsStore();

function onClick() {
  const component = componentsStore.allComponentsById[props.runner.componentId];
  if (component) {
    viewStore.setSelectedComponentId(props.runner.componentId);
    componentsStore.eventBus.emit("panToComponent", {
      component,
      center: true,
    });
    onHoverEnd();
  }
}

const isHover = computed(() => viewStore.hoveredComponentId === props.runner.componentId);

function onHoverStart() {
  if (componentsStore.allComponentsById[props.runner.componentId]) {
    viewStore.setHoveredComponentId(props.runner.componentId);
  }
}

function onHoverEnd() {
  if (componentsStore.allComponentsById[props.runner.componentId]) {
    viewStore.setHoveredComponentId(null);
  }
}

const codeViewerShowing = ref(false);
const toggleCodeViewerShowing = () => {
  codeViewerShowing.value = !codeViewerShowing.value;
};
</script>
