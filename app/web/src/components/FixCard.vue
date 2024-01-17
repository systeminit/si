<template>
  <div
    :class="`flex flex-col items-start p-xs ml-lg bg-white dark:bg-neutral-800 text-sm border-neutral-200 dark:border-neutral-600 overflow-hidden ${
      props.hideTopBorder ? '' : 'border-t'
    }`"
  >
    <div class="flex flex-row items-center text-sm overflow-hidden w-full">
      <StatusIndicatorIcon type="fix" :status="fix.status" />
      <div class="flex flex-col pl-xs flex-shrink overflow-hidden flex-grow">
        <div>{{ `${fix.displayName}` }}</div>
        <div
          ref="componentNameRef"
          v-tooltip="componentNameTooltip"
          :class="
            clsx(
              'truncate cursor-pointer',
              componentsStore.componentsById[fix.componentId]
                ? 'dark:text-action-300 text-action-500 hover:underline font-bold'
                : 'text-neutral-500 dark:text-neutral-400 line-through',
              isHover && 'underline',
            )
          "
          @click="onClick"
          @mouseenter="onHoverStart"
          @mouseleave="onHoverEnd"
        >
          {{ `${fix.componentName}` }}
        </div>
      </div>

      <div
        v-if="fix.resource"
        class="dark:text-action-300 text-action-500 flex-none cursor-pointer"
      >
        <FixDetails
          v-if="fix.resource.logs && !fix.resource?.data"
          :health="fix.resource.status"
          :message="
            [
              `${formatTitle(fix.actionKind)} ${fix.schemaName}`,
              fix.resource.message ??
                (fix.resource.status === 'ok'
                  ? 'Completed successfully'
                  : 'Error'),
            ].filter((f) => f.length > 0)
          "
          :details="fix.resource.logs"
        />
        <FixCardIconButton
          v-else
          tooltip="show code"
          rotate="down"
          icon="code-pop"
          iconHover="code-pop-square"
          :selected="codeViewerShowing"
          @click="toggleCodeViewerShowing"
        />
      </div>
    </div>

    <div
      v-if="codeViewerShowing && fix.resource?.data"
      class="relative w-full mt-xs"
    >
      <CodeViewer
        :code="JSON.stringify(fix.resource.data, null, 2)"
        class="dark:text-neutral-50 text-neutral-900"
      >
        <template #title>
          <div class="font-bold">
            {{ fix.resource.message ?? "Resource Code" }}
            <FixDetails
              v-if="fix.resource.logs && fix.resource.logs.length > 0"
              :health="fix.resource.status"
              :message="
                [
                  `${formatTitle(fix.actionKind)} ${fix.schemaName}`,
                  fix.resource.message ?? '',
                ].filter((f) => f.length > 0)
              "
              :details="fix.resource.logs"
            />
          </div>
        </template>
      </CodeViewer>
    </div>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { PropType, computed, ref } from "vue";
import { Icon, VButton } from "@si/vue-lib/design-system";
import { Fix } from "@/store/fixes.store";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeViewer from "./CodeViewer.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FixDetails from "./FixDetails.vue";
import FixCardIconButton from "./FixCardIconButton.vue";

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();

const props = defineProps({
  fix: { type: Object as PropType<Fix>, required: true },
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
    if (!componentsStore.componentsById[props.fix.componentId]) {
      return {
        content: `Component "${
          componentNameRef.value.textContent
        }" does not exist ${
          changeSetsStore.headSelected ? "on head" : "in this change set"
        }.`,
        delay: { show: 700, hide: 10 },
      };
    } else if (
      componentNameRef.value.scrollWidth > componentNameRef.value.offsetWidth
    ) {
      return {
        content: componentNameRef.value.textContent,
        delay: { show: 700, hide: 10 },
      };
    }
  }
  return {};
});

function onClick() {
  if (componentsStore.componentsById[props.fix.componentId]) {
    componentsStore.setSelectedComponentId(props.fix.componentId);
    componentsStore.eventBus.emit("panToComponent", {
      componentId: props.fix.componentId,
      center: true,
    });
    onHoverEnd();
  }
}

const isHover = computed(
  () => componentsStore.hoveredComponentId === props.fix.componentId,
);

function onHoverStart() {
  if (componentsStore.componentsById[props.fix.componentId]) {
    componentsStore.setHoveredComponentId(props.fix.componentId);
  }
}

function onHoverEnd() {
  if (componentsStore.componentsById[props.fix.componentId]) {
    componentsStore.setHoveredComponentId(null);
  }
}

const codeViewerShowing = ref(false);
const toggleCodeViewerShowing = () => {
  codeViewerShowing.value = !codeViewerShowing.value;
};
</script>
