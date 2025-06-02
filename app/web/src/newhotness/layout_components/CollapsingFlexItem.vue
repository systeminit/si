<template>
  <!-- requires that its parent container is flex (either direction) -->
  <div
    ref="scrollableDivRef"
    :class="
      clsx(
        'collapsing-flex-item', // identifying class
        'border-2 basis-0', // basis-0 makes items take equal size when multiple are open
        themeClasses('border-neutral-200', 'border-neutral-900'),
        openState.open.value ? 'scrollable grow' : 'shrink',
      )
    "
  >
    <h3
      :class="
        clsx(
          'flex flex-row items-center gap-xs p-2xs z-30',
          'sticky top-0 cursor-pointer text-lg font-bold',
          themeClasses('bg-neutral-200', 'bg-neutral-900'),
          h3class,
        )
      "
      @click="toggleOpen"
    >
      <slot name="header" />
      <template v-if="expandable">
        <IconButton
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
        <slot name="header" />
      </template>
      <slot />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses, IconButton, Modal } from "@si/vue-lib/design-system";
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
    h3class: tw`flex flex-row items-center`,
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
