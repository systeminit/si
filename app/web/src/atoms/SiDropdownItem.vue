<template>
  <MenuItem v-slot="{ active }">
    <a
      :class="[
        active ? 'bg-action-500' : '',
        'h-10 flex flex-row relative items-center whitespace-nowrap p-2 cursor-pointer',
        classes,
      ]"
      @click="emit('select')"
    >
      <div class="w-5 mr-2 flex-none" :class="showPrefix ? '' : 'hidden'">
        <slot name="prefix">
          <Icon v-if="checked" name="check" size="sm" />
        </slot>
      </div>
      <div class="min-w-0 text-ellipsis overflow-hidden">
        <slot />
      </div>
      <div class="w-5 ml-2" :class="showSuffix ? '' : 'hidden'">
        <slot name="suffix" />
      </div>
    </a>
  </MenuItem>
</template>

<script setup lang="ts">
import { MenuItem } from "@headlessui/vue";
import { inject } from "vue";
import Icon from "@/ui-lib/icons/Icon.vue";

const emit = defineEmits(["select"]);

// NOTE(nick): below is the divider...
// <div class="w-full h-0.5 my-2 bg-neutral-800" />

const props = defineProps<{
  checked?: boolean;
}>();

const classes = inject("dropdownItemClasses", "text-center");
const showPrefix = inject("dropdownItemShowPrefix", true);
const showSuffix = inject("dropdownItemShowSuffix", true);
</script>
