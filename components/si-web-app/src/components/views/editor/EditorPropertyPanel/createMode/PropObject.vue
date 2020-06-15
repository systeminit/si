<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div
      v-for="field of propObject.properties.attrs.filter(i => !i.hidden)"
      v-bind:key="field.name"
      class="flex flex-row"
    >
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

//@ts-ignore
export default Vue.extend({
  name: "PropObjectView",
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
      try {
        console.log("PropObject.methods.propChangeMsg() with :: ", event);
        this.objectModel[event["fieldName"]] = event["value"];
        this.$emit("propChangeMsg", {
          fieldName: this.propObject.name,
          value: this.objectModel,
        });
      } catch (err) {
        console.log("err: ", err);
      }

      console.log("PropObject.methods.propChangeMsg() completed");
    },
    mounted() {
      console.log("PropObject Mounted");
    },
  },
  data() {
    return {
      objectModel: this.propObjectModel,
    };
  },
});
</script>
