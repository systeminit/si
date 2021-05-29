<template>
  <div class="flex flex-row items-center py-1" v-if="value">
    <div class="flex flex-row justify-end leading-tight text-white w-44">
      {{ label }}
    </div>
    <div
      class="flex flex-grow ml-2 overflow-x-auto leading-tight text-gray-400 select-text"
    >
      {{ value }}
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import _ from "lodash";

export default Vue.extend({
  name: "ResourceField",
  props: {
    label: {
      type: String,
    },
    value: {
      type: [String, Number],
    },
  },
  computed: {
    show(): boolean {
      return _.isUndefined(this.value);
    },
  },
});
</script>
