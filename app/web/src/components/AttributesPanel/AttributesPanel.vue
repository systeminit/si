<template>
  <div
    ref="rootRef"
    :class="
      clsx(
        'attributes-panel',
        showSectionToggles && '--show-section-toggles',
        'flex flex-col min-h-full overflow-hidden',
      )
    "
    @mouseleave="onMouseLeave"
    @pointermove="onMouseMove"
  >
    <ScrollArea :class="themeClasses('bg-neutral-100', 'bg-neutral-900')">
      <template #top>
        <!-- custom inputs for SI props (name, color, etc) -->
        <AttributesPanelCustomInputs />
      </template>

      <LoadingMessage v-if="loadSchemaReqStatus.isPending && !domainTree">
        Loading asset schema
      </LoadingMessage>
      <div
        v-else-if="
          domainTree &&
          domainTree.children.length === 0 &&
          secretsTree &&
          secretsTree.children.length === 0
        "
        :class="
          clsx(
            'text-center px-xs py-sm italic',
            themeClasses('text-neutral-700', 'text-neutral-300'),
          )
        "
      >
        No attributes to display.
      </div>
      <div
        v-else-if="domainTree"
        :class="
          clsx(
            'attributes-panel__items-wrap',
            'relative grow',
            `before:absolute before:w-6 before:left-0 before:top-0 before:bottom-0 before:content-['']`,
            themeClasses(
              'before:bg-neutral-100 bg-shade-0',
              'before:bg-neutral-900 bg-neutral-800',
            ),
          )
        "
      >
        <TreeFormItem
          v-if="domainTree && domainTree.children.length"
          attributesPanel
          :treeDef="domainTree"
          isRootProp
          :context="useAttributesPanelContext"
        />
        <TreeFormItem
          v-if="secretsTree && secretsTree.children.length"
          attributesPanel
          :treeDef="secretsTree"
          isRootProp
          :context="useAttributesPanelContext"
        />
        <TreeFormItem
          v-if="resourceValueTree && resourceValueTree.children.length"
          attributesPanel
          :treeDef="resourceValueTree"
          isRootProp
          startClosed
          :context="useAttributesPanelContext"
        />
      </div>

      <div v-if="SHOW_DEBUG_TREE" class="mt-xl">
        <JsonTreeExplorer :object="domainTree" />
      </div>
    </ScrollArea>
  </div>
</template>

<script lang="ts">
type EventBusEvents = { toggleAllOpen: boolean };

type AttributesPanelContext = {
  eventBus: Emitter<EventBusEvents>;
  hoverSectionValueId: Ref<string | undefined>;
  showSectionToggles: Ref<boolean>;
};

export const AttributesPanelContextInjectionKey: InjectionKey<AttributesPanelContext> =
  Symbol("AttributesPanelContext");

export function useAttributesPanelContext() {
  const ctx = inject(AttributesPanelContextInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<TreeFormItem> requires a context from an <AttributesPanel> or a <TreeForm>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import * as _ from "lodash-es";
import { InjectionKey, Ref, computed, inject, provide, ref } from "vue";
import mitt, { Emitter } from "mitt";
import clsx from "clsx";
import {
  JsonTreeExplorer,
  LoadingMessage,
  ScrollArea,
  themeClasses,
} from "@si/vue-lib/design-system";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { useViewsStore } from "@/store/views.store";
import TreeFormItem from "./TreeFormItem.vue";
import AttributesPanelCustomInputs from "./AttributesPanelCustomInputs.vue";

const rootRef = ref<HTMLDivElement>();

// toggle to true to show JSON tree explorer or attributes/values
const SHOW_DEBUG_TREE = false;

// NON-REACTIVE component id. This works because the parent has a :key which rerenders if the selected component changes
const viewStore = useViewsStore();
const componentId = viewStore.selectedComponent?.def.id;
if (!componentId) {
  throw new Error("Do not use this component without a selectedComponentId");
}

const attributesStore = useComponentAttributesStore(componentId || "NONE");

const loadSchemaReqStatus = attributesStore.getRequestStatus(
  "FETCH_PROPERTY_EDITOR_SCHEMA",
);

const domainTree = computed(() => attributesStore.domainTree);
const secretsTree = computed(() => attributesStore.secretsTree);
const resourceValueTree = computed(() => attributesStore.resourceValueTree);

const showSectionToggles = ref(false);
function onMouseMove(e: PointerEvent) {
  const rect = rootRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left; // x position within the root div
  showSectionToggles.value = x >= 0 && x <= 24;
}
function onMouseLeave() {
  showSectionToggles.value = false;
}

// EXPOSED TO CHILDREN
const eventBus = mitt<EventBusEvents>();
const hoverSectionValueId = ref<string>();

provide(AttributesPanelContextInjectionKey, {
  eventBus,
  hoverSectionValueId,
  showSectionToggles,
});
</script>
