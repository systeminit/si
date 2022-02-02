<template>
  <VueFinalModal
    v-slot="{ close }"
    v-bind="$attrs"
    classes="flex justify-center items-center"
    content-class="relative flex flex-col max-h-full mx-4 p-4 border dark:border-gray-800 rounded bg-gray-900 min-wid"
  >
    <span class="mr-8 text-2xl font-bold">
      <slot name="title"></slot>
    </span>
    <div class="flex-grow overflow-y-auto">
      <slot name="body"></slot>
    </div>
    <div class="flex-shrink-0 flex justify-center items-center pt-4">
      <slot name="buttons">
        <SiButton
          id="modalCancel"
          size="xs"
          label="cancel"
          :icon="null"
          kind="cancel"
          @click="$emit('confirm', close)"
        />
        <SiButton
          id="modalConfirm"
          size="xs"
          label="confirm"
          class="ml-1"
          :icon="null"
          kind="save"
          @click="$emit('cancel', close)"
        />
      </slot>
    </div>
    <button class="absolute top-0 right-0 mt-2 mr-2" @click="close">
      <VueFeather type="x" />
    </button>
  </VueFinalModal>
</template>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>

<script setup lang="ts">
import VueFeather from "vue-feather";
import SiButton from "@/atoms/SiButton.vue";
// @ts-ignore
import { VueFinalModal } from "vue-final-modal";

defineEmits(["confirm", "cancel"]);
</script>
