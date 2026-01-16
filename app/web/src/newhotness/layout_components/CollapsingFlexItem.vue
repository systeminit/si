<template>
  <!-- requires that its parent container is flex (either direction) -->
  <div
    :class="
      clsx(
        'collapsing-flex-item', // identifying class
        'flex flex-col items-stretch',
        'border overflow-hidden mb-[-1px]',
        showOpen ? 'grow' : 'shrink',
        maxHeightContent && 'max-h-fit',
        themeClasses('border-neutral-400', 'border-neutral-600'),
        variant === 'standard' && [
          themeClasses('border-neutral-400', 'border-neutral-600'),
          'basis-0', // makes each item in a group take the same amount of space
        ],
        variant === 'onboarding' && 'min-h-fit rounded-sm bg-[#00000033]',
      )
    "
    :style="variant === 'standard' ? `min-height: ${headerHeight}px` : undefined"
  >
    <h3
      ref="headerRef"
      :class="
        clsx(
          h3class,
          'group/header',
          'flex-none flex items-center px-xs m-0 min-h-[2.5rem]',
          !disableCollapse && [
            'cursor-pointer',
            variant === 'standard' && themeClasses('hover:bg-neutral-100', 'hover:bg-neutral-700'),
          ],
          `text-${computedHeaderTextSize}`,
          variant === 'standard' && [
            showOpen && 'border-b',
            themeClasses('bg-white border-neutral-400', 'bg-neutral-800 border-neutral-600'),
          ],
          variant === 'onboarding' && 'hover:underline',
        )
      "
      @click="toggleOpen"
    >
      <CollapseExpandChevron v-if="!disableCollapse" :open="showOpen" />
      <slot name="header" />
      <div v-if="$slots.headerIcons || showExpandButton" class="ml-auto" />
      <slot name="headerIcons" />
      <NewButton
        v-if="showExpandButton"
        tooltip="Expand"
        tooltipPlacement="top"
        icon="maximize"
        tone="empty"
        size="xs"
        :class="clsx('active:bg-white active:text-black', themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'))"
        @click.prevent.stop="expand"
      />
    </h3>
    <div
      v-if="showOpen"
      :class="
        clsx(
          'min-h-0 flex-1',
          variant === 'standard' && 'scrollable',
          variant === 'onboarding' && 'p-xs pl-lg pt-0 flex flex-col gap-sm text-sm',
        )
      "
    >
      <slot />
    </div>
    <Modal ref="modalRef" size="4xl">
      <template #title>
        <div :class="clsx(h3class)">
          <slot name="header" />
        </div>
      </template>
      <template #titleIcons>
        <slot name="headerIcons" />
      </template>
      <div class="scrollable max-h-[70vh]">
        <slot />
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses, Modal, SpacingSizes, NewButton, CollapseExpandChevron } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, onMounted, ref } from "vue";
import { tw } from "@si/vue-lib";
import { useToggle } from "../logic_composables/toggle_containers";

type CollapsingVariant = "standard" | "onboarding";

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
    maxHeightContent?: boolean;
    variant?: CollapsingVariant;
  }>(),
  {
    h3class: tw`flex flex-row items-center gap-xs p-2xs z-30`,
    expandable: true,
    disableCollapse: false,
    variant: "standard",
  },
);

const headerRef = ref<HTMLDivElement>();
const headerHeight = computed(() => headerRef.value?.offsetHeight ?? 0);

const showOpen = computed(() => openState.open.value || props.disableCollapse);

const computedHeaderTextSize = computed(() => {
  if (props.headerTextSize) return props.headerTextSize;
  else if (props.variant === "onboarding") return "sm";
  else return "lg";
});

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

const showExpandButton = computed(
  () => props.expandable && showOpen.value && !props.disableCollapse && props.variant === "standard",
);

defineExpose({
  openState,
});
</script>
