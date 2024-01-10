<template>
  <!-- <Collapsible
    v-if="fix.resource"
    :defaultOpen="false"
    buttonClasses="bg-white dark:bg-neutral-800 ml-sm"
    contentClasses="flex flex-col items-start p-xs ml-sm bg-white dark:bg-neutral-800 text-sm border-neutral-200 dark:border-neutral-600"
    hideBottomBorderWhenOpen
    extraBorderAtBottomOfContent
  >
    <template #label>
      <div class="flex flex-row items-center text-sm">
        <StatusIndicatorIcon type="fix" :status="fix.status" />
        <div class="flex flex-col pl-xs flex-shrink overflow-hidden">
          <div>{{ `${fix.displayName}` }}</div>
          <div
            ref="componentNameRef"
            v-tooltip="componentNameTooltip"
            :class="
              clsx(
                'text-neutral-500 dark:text-neutral-400 truncate cursor-pointer',
                componentsStore.componentsById[fix.componentId]
                  ? 'dark:hover:text-action-300 hover:text-action-500'
                  : 'line-through',
                isHover && 'dark:text-action-300 text-action-500',
              )
            "
            @click="onClick"
            @mouseenter="onHoverStart"
            @mouseleave="onHoverEnd"
          >
            {{ `${fix.componentName}` }}
          </div>
        </div>
      </div>
    </template>

    <template #default>
      <div class="dark:text-neutral-50 text-neutral-900">
        <CodeViewer
          v-if="fix.resource.data"
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
        <div v-else-if="fix.resource.message" class="text-sm">
          {{ fix.resource.message }}
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
        <div v-else class="text-sm">
          {{
            fix.resource.status === "ok" ? "Completed successfully" : "Error"
          }}
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
      </div>
    </template>
  </Collapsible> -->
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
              'text-neutral-500 dark:text-neutral-400 truncate cursor-pointer',
              componentsStore.componentsById[fix.componentId]
                ? 'dark:hover:text-action-300 hover:text-action-500'
                : 'line-through',
              isHover && 'dark:text-action-300 text-action-500',
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
        class="text-neutral-400 dark:hover:text-action-300 hover:text-action-500 flex-none w-6 h-6 cursor-pointer"
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
        <Icon v-else name="code-circle" @click="toggleCodeViewerShowing" />
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
import { Icon } from "@si/vue-lib/design-system";
import { Fix } from "@/store/fixes.store";
import { useComponentsStore } from "@/store/components.store";
import CodeViewer from "./CodeViewer.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FixDetails from "./FixDetails.vue";

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
    const tooltipText = componentsStore.componentsById[props.fix.componentId]
      ? componentNameRef.value.textContent
      : `Component "${componentNameRef.value.textContent}" does not exist in this change set.`;

    return {
      content: tooltipText,
      delay: { show: 700, hide: 10 },
    };
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
