<template>
  <!-- requires that its parent container is flex (either direction) -->
  <div
    :class="
      clsx(
        'collapsing-flex-item', // identifying class
        'flex flex-col items-stretch',
        'border overflow-hidden basis-0 mb-[-1px]', // basis-0 makes items take equal size when multiple are open
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-800',
        ),
        showOpen ? 'grow' : 'shrink',
      )
    "
    :style="`min-height: ${headerHeight}px`"
  >
    <h3
      ref="headerRef"
      :class="
        clsx(
          h3class,
          'group/header',
          'flex-none flex items-center px-xs m-0',
          !disableCollapse && [
            'cursor-pointer',
            themeClasses('hover:bg-neutral-100', 'hover:bg-neutral-700'),
          ],
          `text-${headerTextSize}`,
          showOpen && 'border-b',
          themeClasses(
            'bg-white border-neutral-400',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
      @click="toggleOpen"
    >
      <Icon
        v-if="!disableCollapse"
        :name="showOpen ? 'chevron-down' : 'chevron-right'"
        size="sm"
      />
      <slot name="header" />
      <div class="ml-auto" />
      <slot name="headerIcons" />
      <IconButton
        v-if="expandable && showOpen && !disableCollapse"
        tooltip="Expand"
        tooltipPlacement="top"
        size="xs"
        icon="maximize"
        iconTone="shade"
        @click.prevent.stop="expand"
      />
    </h3>
    <div v-if="showOpen" :class="clsx('scrollable min-h-0 flex-1')">
      <slot />
    </div>
    <Modal ref="modalRef" size="4xl">
      <template #title>
        <div :class="clsx(h3class)">
          <slot name="header" />
        </div>
      </template>
      <div class="scrollable max-h-[70vh]">
        <slot />
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import {
  themeClasses,
  IconButton,
  Modal,
  Icon,
  SpacingSizes,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, onMounted, ref } from "vue";
import { tw } from "@si/vue-lib";
import { useToggle } from "../logic_composables/toggle_containers";

const openState = useToggle();

const modalRef = ref<InstanceType<typeof Modal>>();

const expand = () => {
  modalRef.value?.open();
};

const props = withDefaults(
  defineProps<{
    open?: boolean;
    h3class?: string;
    expandable?: boolean;
    headerTextSize?: SpacingSizes;
    disableCollapse?: boolean;
  }>(),
  {
    h3class: tw`flex flex-row items-center gap-xs p-2xs z-30`,
    expandable: true,
    disableCollapse: false,
    headerTextSize: "lg",
  },
);

const headerRef = ref<HTMLDivElement>();
const headerHeight = computed(() => headerRef.value?.offsetHeight ?? 0);

const showOpen = computed(() => openState.open.value || props.disableCollapse);

onMounted(() => {
  openState.open.value = props.open;
});

const emit = defineEmits<{
  (e: "toggle"): void;
}>();

const toggleOpen = () => {
  emit("toggle");
  openState.toggle();
};

defineExpose({
  openState,
});
</script>
