<template>
  <!-- requires that its parent container is flex (either direction) -->
  <div
    ref="scrollableDivRef"
    :class="
      clsx(
        'collapsing-flex-item', // identifying class
        'border basis-0 mb-[-1px]', // basis-0 makes items take equal size when multiple are open
        themeClasses('border-neutral-300', 'border-neutral-700'),
        openState.open.value ? 'scrollable grow' : 'shrink',
      )
    "
  >
    <!-- TODO(Wendy) - fix this so that the scrollbar doesn't include the header -->
    <h3
      :class="
        clsx(
          h3class,
          'group/header',
          'sticky top-0 cursor-pointer text-lg font-bold',
          themeClasses(
            'bg-neutral-200 hover:bg-neutral-300',
            'bg-neutral-800 hover:bg-neutral-700',
          ),
        )
      "
      @click="toggleOpen"
    >
      <Icon
        class="group-hover/header:scale-125"
        :name="openState.open.value ? 'chevron--down' : 'chevron--right'"
      />
      <slot name="header" />
      <template v-if="expandable">
        <IconButton
          tooltip="Expand"
          tooltipPlacement="top"
          class="ml-auto"
          size="xs"
          icon="arrows-out"
          iconTone="shade"
          @click.prevent.stop="expand"
        />
      </template>
    </h3>
    <!-- only show contents when open, this makes flexbox grow/shrink work :chefskiss: -->
    <slot v-if="openState.open.value" />

    <Modal ref="modalRef" size="4xl">
      <template #title>
        <div :class="h3class">
          <slot name="header" />
        </div>
      </template>
      <slot />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import {
  themeClasses,
  IconButton,
  Modal,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { onMounted, ref } from "vue";
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
  }>(),
  {
    h3class: tw`flex flex-row items-center gap-xs p-2xs z-30`,
    expandable: true,
  },
);

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
