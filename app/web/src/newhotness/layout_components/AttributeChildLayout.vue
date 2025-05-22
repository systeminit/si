<template>
  <!-- This represents the header & container for component attributes page -->
  <dl class="ml-xs border-l-2 my-xs">
    <!-- this is the left indent & line -->
    <dt
      :class="
        clsx(
          'p-xs flex flex-row items-center cursor-pointer',
          themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
      @click="() => (open = !open)"
    >
      <slot name="header" />
    </dt>
    <dd v-if="open">
      <slot />
      <!-- the children are, so far, another list or a button that would create a list -->
    </dd>
  </dl>
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    defaultOpen?: boolean;
  }>(),
  {
    defaultOpen: true,
  },
);

const open = ref<boolean>(props.defaultOpen);
</script>
