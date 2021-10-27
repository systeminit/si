<template>
  <div class="flex flex-row items-center py-1" v-if="value.length > 0">
    <div class="flex flex-row justify-end leading-tight text-white w-44">
      {{ label }}
    </div>
    <div class="flex flex-grow ml-2 text-gray-400 select-text hover-scrollbar">
      <div class="flex flex-col py-1" v-if="noLabels">
        <div class="flex leading-tight" v-for="i in value" :key="i.value">
          {{ i.value }}
        </div>
      </div>
      <div class="flex flex-col" v-else>
        <template v-for="(i, index) in value" :key="index">
          <ResourceField :label="i.label" :value="i.value"/>
        </template>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import ResourceField from "./ResourceField.vue";

export type ResourceFieldArrayValue = { label: string; value: string }[];


const component = {
  extends: Vue,
  name: "ResourceFieldArray",
  components: {
    ResourceField,
  },
  props: {
    label: {
      type: String,
    },
    value: {
      type: Array as PropType<{ label: string; value: string }[]>,
    },
    noLabels: {
      type: Boolean,
    },
  },
};

export default component
</script>

<style scoped>
.hover-scrollbar {
  overflow: hidden;
}
.hover-scrollbar:hover {
  overflow-x: auto;
}

.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}
</style>
