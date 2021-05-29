<template>
  <div class="flex flex-row items-center w-full py-1">
    <div class="flex flex-row justify-end leading-tight text-white w-44">
      {{ label }}
    </div>
    <div
      class="flex flex-col flex-grow w-full ml-2 mr-4 overflow-x-auto leading-tight text-gray-400 select-text"
    >
      <slot />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
//;import ResourceField from "./ResourceField.vue";

export type ResourceFieldArrayValue = { label: string; value: string }[];

export default Vue.extend({
  name: "ResourceFieldSlot",
  components: {},
  props: {
    label: {
      type: String,
    },
  },
});
</script>
