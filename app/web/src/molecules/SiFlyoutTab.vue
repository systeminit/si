<template>
  <Popover v-slot="{ open }" class="relative">
    <PopoverButton
      :class="[
        open ? 'text-gray-300' : 'text-gray-300',
        'h-12 px-10 group rounded-sm border-black border-l-2 inline-flex items-center text-base font-medium hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-600',
      ]"
    >
      <div v-if="props.kind === SiFlyoutKind.ChangeSet">
        <ClockIcon class="h-7 w-7 mr-1" />
      </div>
      <div v-else-if="props.kind === SiFlyoutKind.Qualifications">
        <CheckCircleIcon class="h-7 w-7 mr-1 text-green-600" />
      </div>

      <span v-if="props.kind === SiFlyoutKind.ChangeSet">Change</span>
      <span v-else-if="props.kind === SiFlyoutKind.Qualifications"
        >Qualifications</span
      >
      <span v-else>Flyout kind not yet implemented</span>
    </PopoverButton>

    <transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-1"
    >
      <SiFlyoutMenu />
    </transition>
  </Popover>
</template>

<script setup lang="ts">
import { Popover, PopoverButton } from "@headlessui/vue";
import { ClockIcon } from "@heroicons/vue/outline";
import { CheckCircleIcon } from "@heroicons/vue/solid";
import SiFlyoutMenu from "@/atoms/SiFlyoutMenu.vue";
import { SiFlyoutKind } from "@/molecules/SiFlyoutTab/types";

const props = defineProps<{
  kind: SiFlyoutKind;
}>();
</script>
