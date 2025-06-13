<template>
  <!-- This represents the header & container for component attributes page -->
  <dl
    :class="
      clsx(
        'ml-xs border-l border-b my-xs',
        themeClasses('border-neutral-200', 'border-neutral-800'),
      )
    "
  >
    <!-- this is the left indent & line -->
    <dt
      :class="
        clsx(
          'px-2xs py-xs flex flex-row items-center gap-2xs cursor-pointer',
          themeClasses(
            'bg-neutral-200 hover:bg-neutral-300',
            'bg-neutral-800 hover:bg-neutral-700',
          ),
        )
      "
      @click="() => (open = !open)"
    >
      <Icon :name="open ? 'chevron--down' : 'chevron--right'" />
      <slot name="header" />
    </dt>
    <dd v-if="open">
      <slot />
      <!-- the children are, so far, another list or a button that would create a list -->
    </dd>
  </dl>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
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
