<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div v-for="field of propObject.properties.attrs.filter(i => !i.hidden)" v-bind:key="field.name" class="flex flex-row">
      <!-- {{objectModel[field.name]}} -->


      <div v-if="typeof propObjectModel[field.name] !== 'undefined'">
        <PropObjectPropertyView
          :propObject="propObject"
          :propObjectProperty="field"
          :propObjectPropertyModel="propObjectModel[field.name]"
        />
      </div>

      <div v-else class="text-red-800">
        propObjectModel[field.name] is undefined
      </div>
      
    </div>
  </div>
</template>

<script lang="ts">
/* eslint-disable vue/no-unused-components */
import Vue from "vue";
import { registry } from "si-registry";

import PropObjectPropertyView from "./PropObjectPropertyView.vue";

import { auth } from "@/utils/auth";

//@ts-ignore
export default Vue.extend({
  name: "PropObject",
  props: {
    propObject: { type: Object, required: true },
    propObjectModel: { type: [Object, Array], required: true },
  },
  components: {
    PropObjectPropertyView,
  },
  methods: {
    mounted(){
      console.log("PropObject Mounted")
    }
  }
});
</script>