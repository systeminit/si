<template>
  <div>
    <div
      v-for="field of propObject.properties.attrs.filter(i => !i.hidden)"
      v-bind:key="field.name"
      class="border border-red-500"
    >
      <div class="px-2 py-2 text-gray-400">
        {{ field.name }}
      </div>

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
import Vue from "vue";
import { registry } from "si-registry";

//import { LinkIcon } from "vue-feather-icons";
import PropObjectProperty from "./PropObjectProperty.vue";

import { auth } from "@/auth";

//@ts-ignore
export default Vue.extend({
  name: "PropObject",
  props: {
    propObject: { type: Object, required: true },
    propObjectModel: { type: [Object, Array], required: true },
  },
  components: {
    //    LinkIcon,
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
  },
  data() {
    return {
      objectModel: this.propObjectModel,
    };
  },
});
</script>

<style>
.property-editor-bg-color {
  background-color: #212324;
}

.input-bg-color {
  background-color: #25788a;
}
</style>
