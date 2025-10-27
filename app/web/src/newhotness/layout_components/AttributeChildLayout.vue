<template>
  <!-- This represents the header & container for component attributes page -->
  <dl
    :class="
      clsx(
        'border',
        sticky ? 'my-0' : 'my-2xs',
        themeClasses('border-neutral-300', 'border-neutral-600'),
      )
    "
  >
    <!-- this is the left indent & line -->
    <dt
      :class="
        clsx(
          'group/header',
          'px-2xs py-xs flex flex-row items-center gap-2xs cursor-pointer h-lg',
          sticky && 'sticky',
          _open && 'border-b',
          themeClasses(
            'bg-white border-neutral-300 hover:bg-neutral-100',
            'bg-neutral-800 border-neutral-600 hover:bg-neutral-700',
          ),
        )
      "
      :style="
        sticky ? { top: `${stickyTopOffset}px`, zIndex: stickyZIndex } : {}
      "
      @click="() => (_open = !_open)"
    >
      <CollapseExpandChevron :open="_open" />
      <slot name="header" />
    </dt>
    <dd v-if="_open" class="p-2xs">
      <slot />
      <!-- the children are, so far, another list or a button that would create a list -->
    </dd>
  </dl>
</template>

<script setup lang="ts">
import { CollapseExpandChevron, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    defaultOpen?: boolean;
    sticky?: boolean;
    stickyTopOffset?: number;
    stickyZIndex?: number;
  }>(),
  {
    defaultOpen: true,
    sticky: false,
    stickyTopOffset: 0,
    stickyZIndex: 10,
  },
);

const close = () => {
  _open.value = false;
};

const open = () => {
  _open.value = true;
};

defineExpose({
  close,
  open,
});

const _open = ref<boolean>(props.defaultOpen);
</script>
