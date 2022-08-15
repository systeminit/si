<template>
  <MenuItem v-slot="{ active }">
    <a
      :class="[
        active ? 'bg-action-500' : '',
        'flex relative items-center text-center whitespace-nowrap p-2 text-shade-0 cursor-pointer',
      ]"
      @click="emit('select')"
    >
      <div class="w-5 mr-2">
        <slot name="prefix">
          <CheckIcon v-if="props.checked" />
        </slot>
      </div>
      <slot />

      <div class="w-5 ml-2">
        <slot name="suffix" />
      </div>
    </a>
  </MenuItem>
</template>

<script setup lang="ts">
import { MenuItem } from "@headlessui/vue";
import { CheckIcon } from "@heroicons/vue/solid";

const emit = defineEmits(["select"]);

// NOTE(nick): below is the divider...
// <div class="w-full h-0.5 my-2 bg-neutral-800" />

const props = defineProps<{
  checked?: boolean;
}>();
</script>
