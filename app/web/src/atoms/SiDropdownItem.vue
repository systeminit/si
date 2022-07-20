<template>
  <MenuItem v-slot="{ active }">
    <a
      :class="[
        active ? 'bg-[#2F80ED]' : '',
        'flex relative items-center text-center whitespace-nowrap px-2 py-2 text-sm text-white cursor-pointer',
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
// <div class="w-full h-0.5 my-2 bg-[#333333]" />

const props = defineProps<{
  checked?: boolean;
}>();
</script>
