<template>
  <div
    ref="rootRef"
    :class="
      clsx(
        'tree-form',
        showSectionToggles && '--show-section-toggles',
        'flex flex-col min-h-full',
      )
    "
    @mouseleave="onMouseLeave"
    @pointermove="onMouseMove"
  >
    <div
      :class="
        clsx(
          'tree-form__items-wrap',
          'relative grow pb-md',
          `before:absolute before:w-6 before:left-0 before:top-0 before:bottom-0 before:content-['']`,
          themeClasses('before:bg-neutral-100', 'before:bg-neutral-900'),
        )
      "
    >
      <TreeFormItem
        v-for="tree in trees"
        :key="tree.propId"
        :treeDef="tree"
        isRootProp
        :context="useAttributesPanelContext"
      />
    </div>
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
import mitt, { Emitter } from "mitt";
import clsx from "clsx";
import { inject, InjectionKey, PropType, provide, Ref, ref } from "vue";
import { themeClasses } from "@si/vue-lib/design-system";
import TreeFormItem, { TreeFormData } from "./TreeFormItem.vue";

defineProps({
  trees: {
    type: Array as PropType<TreeFormData[]>,
    required: true,
  },
});

const rootRef = ref<HTMLDivElement>();

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
