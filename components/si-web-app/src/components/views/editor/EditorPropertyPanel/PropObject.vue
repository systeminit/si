<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div v-for="field of propObject.properties.attrs.filter(i => !i.hidden)" v-bind:key="field.name" class="flex flex-row">
      
      <div class="px-2 py-2 text-gray-400"/>
        <PropObjectProperty
          :propObject="propObject"
          :propObjectProperty="field"
          :propObjectPropertyModel="objectModel[field.name]"
          @propChangeMsg="propChangeMsg"
        />

    </div>
  </div>
</template>

<script lang="ts">
/* eslint-disable vue/no-unused-components */
import Vue from "vue";
import { registry } from "si-registry";

import PropObjectProperty from "./PropObjectProperty.vue";

import { auth } from "@/utils/auth";

//@ts-ignore
export default Vue.extend({
  name: "PropObject",
  props: {
    propObject: { type: Object, required: true },
    propObjectModel: { type: [Object, Array], required: true },
  },
  components: {
 // LinkIcon,
    PropObjectProperty,
  },
  methods: {
    propChangeMsg(event: any) {
      this.objectModel[event["fieldName"]] = event["value"];
      this.$emit("propChangeMsg", {
        fieldName: this.propObject.name,
        value: this.objectModel,
      });
    },
    mounted(){
      console.log("PropObject Mounted")
    }
  },
  data() {
    return {
      objectModel: this.propObjectModel,
    };
  },
});
</script>